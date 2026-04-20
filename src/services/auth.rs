use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::db::{self, DbPool};
use crate::error::AppError;
use crate::models::{RefreshTokenChain, User};
use crate::services::crypto::CryptoService;
use super::metrics::{increment_login_failed, increment_login_success, increment_2fa_attempts, record_login_latency, record_refresh_latency, LatencyTimer};

pub struct AuthService {
    pool: DbPool,
    crypto: Arc<CryptoService>,
    challenge_tokens: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl AuthService {
    pub fn new(pool: DbPool, crypto: Arc<CryptoService>) -> Self {
        Self {
            pool,
            crypto,
            challenge_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<LoginResult, AppError> {
        let _timer = LatencyTimer::new();

        let user = match db::get_user_by_email(&self.pool, email).await? {
            Some(u) => u,
            None => {
                increment_login_failed();
                return Err(AppError::InvalidCredentials);
            }
        };

        if !user.is_active {
            return Err(AppError::UserInactive);
        }

        if !self.crypto.verify_password(password, &user.password_hash)? {
            increment_login_failed();
            return Err(AppError::InvalidCredentials);
        }

        if user.is_2fa_enabled {
            let challenge_token = self.crypto.generate_challenge_token(user.id)?;
            self.challenge_tokens
                .write()
                .await
                .insert(challenge_token.clone(), user.id);

            return Ok(LoginResult::TwoFactorChallenge {
                challenge_token,
                expires_in: 300,
            });
        }

        let roles = db::get_roles_by_user_id(&self.pool, user.id).await?;
        let role = roles.first().map(|r| r.name.as_str()).unwrap_or("User");

        let access_token = self.crypto.generate_access_token(user.id, role)?;
        let refresh_token = self.crypto.generate_refresh_token();
        let refresh_hash = self.crypto.hash_token(&refresh_token);
        let expires_at = self.crypto.calculate_refresh_expiry();
        let chain_id = Uuid::new_v4();

        db::create_refresh_token_chain(
            &self.pool,
            user.id,
            chain_id,
            &refresh_hash,
            None,
            expires_at,
        )
        .await?;

        increment_login_success();
        record_login_latency(_timer.elapsed_ms());

        Ok(LoginResult::Success {
            access_token,
            refresh_token,
            expires_in: 900,
        })
    }

    pub async fn verify_2fa(
        &self,
        challenge_token: &str,
        code: &str,
    ) -> Result<LoginResult, AppError> {
        increment_2fa_attempts();

        let user_id = {
            let tokens = self.challenge_tokens.read().await;
            tokens
                .get(challenge_token)
                .copied()
                .ok_or(AppError::InvalidToken)?
        };

        let user = db::get_user_by_id(&self.pool, user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        let totp_secret = user
            .totp_secret
            .as_ref()
            .ok_or(AppError::InternalError)?;

        if !self.crypto.verify_totp(totp_secret, code)? {
            return Err(AppError::InvalidTwoFactorCode);
        }

        {
            let mut tokens = self.challenge_tokens.write().await;
            tokens.remove(challenge_token);
        }

        let roles = db::get_roles_by_user_id(&self.pool, user.id).await?;
        let role = roles.first().map(|r| r.name.as_str()).unwrap_or("User");

        let access_token = self.crypto.generate_access_token(user.id, role)?;
        let refresh_token = self.crypto.generate_refresh_token();
        let refresh_hash = self.crypto.hash_token(&refresh_token);
        let expires_at = self.crypto.calculate_refresh_expiry();
        let chain_id = Uuid::new_v4();

        db::create_refresh_token_chain(
            &self.pool,
            user.id,
            chain_id,
            &refresh_hash,
            None,
            expires_at,
        )
        .await?;

        increment_login_success();
        record_login_latency(LatencyTimer::new().elapsed_ms());

        Ok(LoginResult::Success {
            access_token,
            refresh_token,
            expires_in: 900,
        })
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<RefreshResult, AppError> {
        let token_hash = self.crypto.hash_token(refresh_token);

        let stored_token = db::get_refresh_token(&self.pool, &token_hash)
            .await?
            .ok_or(AppError::InvalidToken)?;

        if stored_token.is_revoked {
            db::revoke_chain(&self.pool, stored_token.chain_id).await?;
            return Err(AppError::TokenRevoked);
        }

        if stored_token.expires_at < chrono::Utc::now() {
            return Err(AppError::TokenExpired);
        }

        let user = db::get_user_by_id(&self.pool, stored_token.user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        if !user.is_active {
            return Err(AppError::UserInactive);
        }

        let roles = db::get_roles_by_user_id(&self.pool, user.id).await?;
        let role = roles.first().map(|r| r.name.as_str()).unwrap_or("User");

        let access_token = self.crypto.generate_access_token(user.id, role)?;
        let new_refresh_token = self.crypto.generate_refresh_token();
        let new_refresh_hash = self.crypto.hash_token(&new_refresh_token);
        let expires_at = self.crypto.calculate_refresh_expiry();

        db::delete_token(&self.pool, stored_token.id).await?;

        db::create_refresh_token_chain(
            &self.pool,
            user.id,
            stored_token.chain_id,
            &new_refresh_hash,
            Some(&token_hash),
            expires_at,
        )
        .await?;

        Ok(RefreshResult {
            access_token,
            refresh_token: new_refresh_token,
            expires_in: 900,
        })
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        let token_hash = self.crypto.hash_token(refresh_token);

        let stored_token = db::get_refresh_token(&self.pool, &token_hash)
            .await?
            .ok_or(AppError::InvalidToken)?;

        db::revoke_token(&self.pool, stored_token.id).await?;
        Ok(())
    }

    pub async fn setup_2fa(&self, user_id: Uuid) -> Result<TotpSetupResult, AppError> {
        let secret = self.crypto.generate_totp_secret();
        let totp_uri = self.crypto.generate_totp_uri(&secret, &user_id.to_string());

        db::update_user_totp_secret(&self.pool, user_id, &secret).await?;

        Ok(TotpSetupResult { secret, totp_uri })
    }

    pub async fn enable_2fa(&self, user_id: Uuid, code: &str) -> Result<(), AppError> {
        let user = db::get_user_by_id(&self.pool, user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        let totp_secret = user
            .totp_secret
            .as_ref()
            .ok_or(AppError::BadRequest("2FA not setup".to_string()))?;

        if !self.crypto.verify_totp(totp_secret, code)? {
            return Err(AppError::InvalidTwoFactorCode);
        }

        db::enable_user_2fa(&self.pool, user_id).await?;
        Ok(())
    }

    pub async fn password_forgot(&self, email: &str) -> Result<(), AppError> {
        let user = match db::get_user_by_email(&self.pool, email).await? {
            Some(u) => u,
            None => return Ok(()),
        };

        let token = self.crypto.generate_refresh_token();
        let token_hash = self.crypto.hash_token(&token);
        let expires_at = self.crypto.calculate_password_reset_expiry();

        db::create_password_reset(&self.pool, user.id, &token_hash, expires_at).await?;

        tracing::info!(
            "Password reset requested for user {} - token hash: {}",
            user.id,
            token_hash
        );

        Ok(())
    }

    pub async fn password_reset(&self, token: &str, new_password: &str) -> Result<(), AppError> {
        let token_hash = self.crypto.hash_token(token);

        let reset = db::get_password_reset(&self.pool, &token_hash)
            .await?
            .ok_or(AppError::InvalidToken)?;

        if reset.expires_at < chrono::Utc::now() {
            db::delete_password_reset(&self.pool, &token_hash).await?;
            return Err(AppError::TokenExpired);
        }

        let password_hash = self.crypto.hash_password(new_password)?;
        db::update_user_password(&self.pool, reset.user_id, &password_hash).await?;
        db::delete_password_reset(&self.pool, &token_hash).await?;

        Ok(())
    }

    pub async fn create_user(&self, email: &str, password: &str) -> Result<User, AppError> {
        let password_hash = self.crypto.hash_password(password)?;
        let user = db::create_user(&self.pool, email, &password_hash).await?;

        if let Some(admin_role) = db::get_admin_role(&self.pool).await? {
            db::assign_role_to_user(&self.pool, user.id, admin_role.id).await?;
        }

        Ok(user)
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<User, AppError> {
        db::get_user_by_id(&self.pool, user_id)
            .await?
            .ok_or(AppError::UserNotFound)
    }

    pub async fn emergency_recover(&self, email: &str) -> Result<String, AppError> {
        let user = db::get_user_by_email(&self.pool, email)
            .await?
            .ok_or(AppError::UserNotFound)?;

        let token = self.crypto.generate_emergency_token(user.id)?;

        tracing::warn!(
            "Emergency recovery accessed for user {} from IP",
            user.id
        );

        Ok(token)
    }

    pub async fn cleanup_expired_tokens(&self) -> Result<u64, AppError> {
        let count = db::cleanup_expired_tokens(&self.pool).await?;
        tracing::info!("Cleaned up {} expired tokens", count);
        Ok(count)
    }
}

pub enum LoginResult {
    Success {
        access_token: String,
        refresh_token: String,
        expires_in: i64,
    },
    TwoFactorChallenge {
        challenge_token: String,
        expires_in: i64,
    },
}

pub struct RefreshResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub struct TotpSetupResult {
    pub secret: String,
    pub totp_uri: String,
}
