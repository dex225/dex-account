use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    routing::post,
    Json, Router,
};

use crate::error::AppError;
use crate::middleware::auth::UserId;
use crate::models::*;
use crate::services::{AuthService, CryptoService};

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub crypto: Arc<CryptoService>,
    pub emergency_api_key: String,
}

pub fn create_router(
    auth: Arc<AuthService>,
    crypto: Arc<CryptoService>,
    emergency_api_key: String,
) -> Router {
    let state = AppState {
        auth,
        crypto,
        emergency_api_key,
    };

    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/verify-2fa", post(verify_2fa))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/auth/password/forgot", post(password_forgot))
        .route("/auth/password/reset", post(password_reset))
        .route("/auth/2fa/setup", post(setup_2fa))
        .route("/auth/2fa/enable", post(enable_2fa))
        .route("/auth/emergency/recover", post(emergency_recover))
        .route("/users/create", post(create_user))
        .route("/users/me", post(get_me))
        .with_state(state)
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = state.auth.login(&req.email, &req.password).await?;

    match result {
        crate::services::LoginResult::Success {
            access_token,
            expires_in,
            ..
        } => Ok(Json(serde_json::json!({
            "access_token": access_token,
            "token_type": "Bearer",
            "expires_in": expires_in
        }))),
        crate::services::LoginResult::TwoFactorChallenge {
            challenge_token,
            expires_in,
        } => Ok(Json(serde_json::json!({
            "challenge_token": challenge_token,
            "expires_in": expires_in
        }))),
    }
}

async fn verify_2fa(
    State(state): State<AppState>,
    Json(req): Json<VerifyTwoFactorRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = state.auth.verify_2fa(&req.challenge_token, &req.code).await?;

    match result {
        crate::services::LoginResult::Success {
            access_token,
            expires_in,
            ..
        } => Ok(Json(serde_json::json!({
            "access_token": access_token,
            "token_type": "Bearer",
            "expires_in": expires_in
        }))),
        _ => Err(AppError::InternalError),
    }
}

async fn refresh(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> Result<Json<serde_json::Value>, AppError> {
    let refresh_token = extract_refresh_token(&req)?;

    let result = state.auth.refresh(&refresh_token).await?;

    Ok(Json(serde_json::json!({
        "access_token": result.access_token,
        "token_type": "Bearer",
        "expires_in": result.expires_in
    })))
}

async fn logout(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> Result<Json<serde_json::Value>, AppError> {
    let refresh_token = extract_refresh_token(&req)?;
    state.auth.logout(&refresh_token).await?;

    Ok(Json(serde_json::json!({ "message": "Logged out" })))
}

async fn password_forgot(
    State(state): State<AppState>,
    Json(req): Json<PasswordForgotRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.auth.password_forgot(&req.email).await?;
    Ok(Json(serde_json::json!({ "message": "If the email exists, a reset link has been sent" })))
}

async fn password_reset(
    State(state): State<AppState>,
    Json(req): Json<PasswordResetRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.auth.password_reset(&req.token, &req.new_password).await?;
    Ok(Json(serde_json::json!({ "message": "Password reset successfully" })))
}

async fn setup_2fa(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
) -> Result<Json<TwoFactorSetupResponse>, AppError> {
    let result = state.auth.setup_2fa(user_id.0).await?;
    Ok(Json(TwoFactorSetupResponse {
        totp_uri: result.totp_uri,
        secret: result.secret,
    }))
}

async fn enable_2fa(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Json(req): Json<EnableTwoFactorRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.auth.enable_2fa(user_id.0, &req.code).await?;
    Ok(Json(serde_json::json!({ "message": "2FA enabled successfully" })))
}

async fn emergency_recover(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<EmergencyRecoverRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let api_key = headers
        .get("X-Emergency-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    if api_key != state.emergency_api_key {
        return Err(AppError::Forbidden);
    }

    let token = state.auth.emergency_recover(&req.email).await?;
    Ok(Json(serde_json::json!({
        "access_token": token,
        "token_type": "Bearer",
        "expires_in": 300
    })))
}

async fn create_user(
    State(state): State<AppState>,
    Extension(_user_id): Extension<UserId>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.auth.create_user(&req.email, &req.password).await?;
    Ok(Json(UserResponse::from(user)))
}

async fn get_me(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.auth.get_user(user_id.0).await?;
    Ok(Json(UserResponse::from(user)))
}

fn extract_refresh_token(req: &axum::extract::Request) -> Result<String, AppError> {
    req.headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let mut parts = cookie.trim().split('=');
                if parts.next() == Some("refresh_token") {
                    parts.next().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .ok_or(AppError::Unauthorized)
}
