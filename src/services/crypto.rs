use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};
use totp_rs::{Secret, TOTP};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::Claims;

const ACCESS_TOKEN_EXPIRY_MINUTES: i64 = 15;
const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 15;
const CHALLENGE_TOKEN_EXPIRY_MINUTES: i64 = 5;
const PASSWORD_RESET_EXPIRY_MINUTES: i64 = 30;

pub struct CryptoService {
    jwt_secret: String,
    argon2: Argon2<'static>,
}

impl CryptoService {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            argon2: Argon2::default(),
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AppError::InternalError)?;
        Ok(hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|_| AppError::InternalError)?;
        Ok(parsed_hash.verify_password(&[], password).is_ok())
    }

    pub fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        role: &str,
    ) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::minutes(ACCESS_TOKEN_EXPIRY_MINUTES);

        let claims = Claims {
            sub: user_id.to_string(),
            role: role.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AppError::InternalError)
    }

    pub fn generate_challenge_token(&self, user_id: Uuid) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::minutes(CHALLENGE_TOKEN_EXPIRY_MINUTES);

        let claims = Claims {
            sub: user_id.to_string(),
            role: "2fa_challenge".to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AppError::InternalError)
    }

    pub fn validate_challenge_token(&self, token: &str) -> Result<Uuid, AppError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::InvalidToken)?;

        if token_data.claims.role != "2fa_challenge" {
            return Err(AppError::InvalidToken);
        }

        Uuid::parse_str(&token_data.claims.sub).map_err(|_| AppError::InvalidToken)
    }

    pub fn generate_refresh_token(&self) -> String {
        use rand::RngCore;
        let mut bytes = [0u8; 64];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(bytes)
    }

    pub fn calculate_refresh_expiry(&self) -> chrono::DateTime<Utc> {
        Utc::now() + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS)
    }

    pub fn calculate_password_reset_expiry(&self) -> chrono::DateTime<Utc> {
        Utc::now() + Duration::minutes(PASSWORD_RESET_EXPIRY_MINUTES)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            if e.to_string().contains("Expired") {
                AppError::TokenExpired
            } else {
                AppError::InvalidToken
            }
        })
        .map(|data| data.claims)
    }

    pub fn generate_totp_secret(&self) -> String {
        use base32::Alphabet;
        let secret = base32::encode(Alphabet::Rfc4648 { padding: false }, &rand::random::<[u8; 20]>());
        secret
    }

    pub fn generate_totp_uri(&self, secret: &str, email: &str) -> String {
        format!("otpauth://totp/dex-account:{}?secret={}&issuer=dex-account&algorithm=SHA1&digits=6&period=30", email, secret)
    }

    pub fn verify_totp(&self, secret: &str, code: &str) -> Result<bool, AppError> {
        use base32::Alphabet;
        let key = base32::decode(Alphabet::Rfc4648 { padding: false }, secret)
            .ok_or(AppError::InternalError)?;

        let totp = TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            key,
            None,
            "dex-account".to_string(),
        )
        .map_err(|_| AppError::InternalError)?;

        Ok(totp.check_current(code).unwrap_or(false))
    }

    pub fn generate_uuid_v7(&self) -> Uuid {
        Uuid::new_v4()
    }

    pub fn generate_emergency_token(&self, user_id: Uuid) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::minutes(5);

        let claims = Claims {
            sub: user_id.to_string(),
            role: "Admin".to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AppError::InternalError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let service = CryptoService::new("test-secret".to_string());
        let hash = service.hash_password("password123").unwrap();
        assert!(service.verify_password("password123", &hash).unwrap());
        assert!(!service.verify_password("wrongpassword", &hash).unwrap());
    }

    #[test]
    fn test_hash_token() {
        let service = CryptoService::new("test-secret".to_string());
        let hash1 = service.hash_token("token123");
        let hash2 = service.hash_token("token123");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, service.hash_token("token456"));
    }

    #[test]
    fn test_generate_and_validate_access_token() {
        let service = CryptoService::new("test-secret".to_string());
        let user_id = Uuid::new_v4();
        let token = service.generate_access_token(user_id, "Admin").unwrap();
        let claims = service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.role, "Admin");
    }
}
