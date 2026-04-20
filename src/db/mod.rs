use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{PasswordReset, RefreshTokenChain, Role, User};

pub type DbPool = PgPool;

pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn get_user_by_email(pool: &DbPool, email: &str) -> Result<Option<User>, AppError> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, totp_secret, is_2fa_enabled, is_active, created_at, updated_at 
         FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn get_user_by_id(pool: &DbPool, user_id: Uuid) -> Result<Option<User>, AppError> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, totp_secret, is_2fa_enabled, is_active, created_at, updated_at 
         FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn create_user(pool: &DbPool, email: &str, password_hash: &str) -> Result<User, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, password_hash, is_2fa_enabled, is_active, created_at, updated_at)
         VALUES ($1, $2, $3, false, true, $4, $4)
         RETURNING id, email, password_hash, totp_secret, is_2fa_enabled, is_active, created_at, updated_at"
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn get_roles_by_user_id(pool: &DbPool, user_id: Uuid) -> Result<Vec<Role>, AppError> {
    sqlx::query_as::<_, Role>(
        "SELECT r.id, r.name FROM roles r
         INNER JOIN user_roles ur ON r.id = ur.role_id
         WHERE ur.user_id = $1"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn create_refresh_token_chain(
    pool: &DbPool,
    user_id: Uuid,
    chain_id: Uuid,
    token_hash: &str,
    previous_token_hash: Option<&str>,
    expires_at: DateTime<Utc>,
) -> Result<RefreshTokenChain, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query_as::<_, RefreshTokenChain>(
        "INSERT INTO refresh_token_chains (id, user_id, chain_id, token_hash, previous_token_hash, created_at, expires_at, is_revoked)
         VALUES ($1, $2, $3, $4, $5, $6, $7, false)
         RETURNING id, user_id, chain_id, token_hash, previous_token_hash, created_at, expires_at, is_revoked"
    )
    .bind(id)
    .bind(user_id)
    .bind(chain_id)
    .bind(token_hash)
    .bind(previous_token_hash)
    .bind(now)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn get_refresh_token(pool: &DbPool, token_hash: &str) -> Result<Option<RefreshTokenChain>, AppError> {
    sqlx::query_as::<_, RefreshTokenChain>(
        "SELECT id, user_id, chain_id, token_hash, previous_token_hash, created_at, expires_at, is_revoked
         FROM refresh_token_chains
         WHERE token_hash = $1"
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn revoke_token(pool: &DbPool, token_id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE refresh_token_chains SET is_revoked = true WHERE id = $1")
        .bind(token_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn revoke_chain(pool: &DbPool, chain_id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE refresh_token_chains SET is_revoked = true WHERE chain_id = $1")
        .bind(chain_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn delete_token(pool: &DbPool, token_id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM refresh_token_chains WHERE id = $1")
        .bind(token_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn create_password_reset(
    pool: &DbPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<PasswordReset, AppError> {
    sqlx::query_as::<_, PasswordReset>(
        "INSERT INTO password_resets (token_hash, user_id, expires_at)
         VALUES ($1, $2, $3)
         RETURNING token_hash, user_id, expires_at"
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn get_password_reset(pool: &DbPool, token_hash: &str) -> Result<Option<PasswordReset>, AppError> {
    sqlx::query_as::<_, PasswordReset>(
        "SELECT token_hash, user_id, expires_at FROM password_resets WHERE token_hash = $1"
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_password_reset(pool: &DbPool, token_hash: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM password_resets WHERE token_hash = $1")
        .bind(token_hash)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn update_user_password(pool: &DbPool, user_id: Uuid, password_hash: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3")
        .bind(password_hash)
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn update_user_totp_secret(pool: &DbPool, user_id: Uuid, totp_secret: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET totp_secret = $1, updated_at = $2 WHERE id = $3")
        .bind(totp_secret)
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn enable_user_2fa(pool: &DbPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET is_2fa_enabled = true, updated_at = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn cleanup_expired_tokens(pool: &DbPool) -> Result<u64, AppError> {
    let result = sqlx::query("DELETE FROM refresh_token_chains WHERE expires_at < NOW() AND is_revoked = true")
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(result.rows_affected())
}

pub async fn get_admin_role(pool: &DbPool) -> Result<Option<Role>, AppError> {
    sqlx::query_as::<_, Role>("SELECT id, name FROM roles WHERE name = 'Admin'")
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn assign_role_to_user(pool: &DbPool, user_id: Uuid, role_id: Uuid) -> Result<(), AppError> {
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .bind(role_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}
