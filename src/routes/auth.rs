use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    middleware::{from_fn, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum::http::{HeaderName, HeaderValue};

use crate::middleware::client_ip::client_ip_middleware;
use crate::middleware::ClientIp;

const REFRESH_TOKEN_COOKIE: HeaderName = HeaderName::from_static("set-cookie");

fn create_refresh_cookie(token: &str, max_age_secs: u64) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "refresh_token={}; HttpOnly; SameSite=Strict; Secure; Path=/; Max-Age={}",
        token, max_age_secs
    ))
    .expect("Invalid cookie header")
}

use crate::error::AppError;
use crate::middleware::auth::{auth_middleware, AuthState, UserId};
use crate::middleware::ip_lockout::IpLockout;
use crate::middleware::rate_limit::{
    general_rate_limit, login_rate_limit, password_forgot_rate_limit, verify_2fa_rate_limit,
};
use crate::models::*;
use crate::services::{AuthService, CryptoService};

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub crypto: Arc<CryptoService>,
    pub emergency_api_key: String,
    pub setup_token: String,
    pub ip_lockout: IpLockout,
}

pub fn create_router(
    auth: Arc<AuthService>,
    crypto: Arc<CryptoService>,
    emergency_api_key: String,
    setup_token: String,
    ip_lockout: IpLockout,
) -> Router {
    let state = AppState {
        auth,
        crypto: crypto.clone(),
        emergency_api_key,
        setup_token,
        ip_lockout,
    };

    let auth_state = AuthState { crypto };

    Router::new()
        .layer(axum::middleware::from_fn(client_ip_middleware))
        .route("/auth/login", post(login).layer(login_rate_limit()))
        .route("/auth/verify-2fa", post(verify_2fa).layer(verify_2fa_rate_limit()))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/auth/password/forgot", post(password_forgot).layer(password_forgot_rate_limit()))
        .route("/auth/password/reset", post(password_reset))
        .route("/auth/setup", post(setup).layer(general_rate_limit()))
        .route("/auth/2fa/setup", post(setup_2fa).layer(general_rate_limit()).layer(from_fn_with_state(auth_state.clone(), auth_middleware)))
        .route("/auth/2fa/enable", post(enable_2fa).layer(verify_2fa_rate_limit()).layer(from_fn_with_state(auth_state.clone(), auth_middleware)))
        .route("/auth/emergency/recover", post(emergency_recover).layer(general_rate_limit()))
        .route("/users/create", post(create_user).layer(general_rate_limit()).layer(from_fn_with_state(auth_state.clone(), auth_middleware)))
        .route("/users/me", get(get_me).layer(from_fn_with_state(auth_state, auth_middleware)))
        .with_state(state)
}

async fn login(
    State(state): State<AppState>,
    Extension(client_ip): Extension<ClientIp>,
    Json(req): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let ip = client_ip.0.to_string();

    if state.ip_lockout.is_locked(&ip) {
        let remaining = state.ip_lockout.get_remaining_lockout_secs(&ip).unwrap_or(900);
        return Err(AppError::IpLocked(remaining / 60));
    }

    let result = state.auth.login(&req.email, &req.password).await;

    match result {
        Ok(crate::services::LoginResult::Success {
            access_token,
            refresh_token,
            expires_in,
        }) => {
            state.ip_lockout.record_success(&ip);
            let mut response = Json(serde_json::json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": expires_in
            })).into_response();
            response.headers_mut().insert(
                REFRESH_TOKEN_COOKIE,
                create_refresh_cookie(&refresh_token, 1296000),
            );
            Ok(response)
        }
        Ok(crate::services::LoginResult::TwoFactorChallenge {
            challenge_token,
            expires_in,
        }) => Ok(Json(serde_json::json!({
            "challenge_token": challenge_token,
            "expires_in": expires_in
        })).into_response()),
        Err(e) => {
            if matches!(e, AppError::InvalidCredentials) {
                state.ip_lockout.record_failure(&ip);
            }
            Err(e)
        }
    }
}

async fn verify_2fa(
    State(state): State<AppState>,
    Extension(client_ip): Extension<ClientIp>,
    Json(req): Json<VerifyTwoFactorRequest>,
) -> Result<Response, AppError> {
    let ip = client_ip.0.to_string();

    if state.ip_lockout.is_locked(&ip) {
        let remaining = state.ip_lockout.get_remaining_lockout_secs(&ip).unwrap_or(900);
        return Err(AppError::IpLocked(remaining / 60));
    }

    let result = state.auth.verify_2fa(&req.challenge_token, &req.code).await;

    match result {
        Ok(crate::services::LoginResult::Success {
            access_token,
            refresh_token,
            expires_in,
        }) => {
            state.ip_lockout.record_success(&ip);
            let mut response = Json(serde_json::json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": expires_in
            })).into_response();
            response.headers_mut().insert(
                REFRESH_TOKEN_COOKIE,
                create_refresh_cookie(&refresh_token, 1296000),
            );
            Ok(response)
        }
        Err(e) => {
            if matches!(e, AppError::InvalidTwoFactorCode) {
                state.ip_lockout.record_failure(&ip);
            }
            Err(e)
        }
        _ => Err(AppError::InternalError),
    }
}

async fn refresh(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> Result<Response, AppError> {
    let refresh_token = extract_refresh_token(&req)?;

    let result = state.auth.refresh(&refresh_token).await?;

    let mut response = Json(serde_json::json!({
        "access_token": result.access_token,
        "token_type": "Bearer",
        "expires_in": result.expires_in
    })).into_response();
    response.headers_mut().insert(
        REFRESH_TOKEN_COOKIE,
        create_refresh_cookie(&result.refresh_token, 1296000),
    );
    Ok(response)
}

async fn logout(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> Result<Response, AppError> {
    let refresh_token = extract_refresh_token(&req)?;
    state.auth.logout(&refresh_token).await?;

    let mut response = Json(serde_json::json!({ "message": "Logged out" })).into_response();
    response.headers_mut().insert(
        REFRESH_TOKEN_COOKIE,
        HeaderValue::from_str("refresh_token=; HttpOnly; SameSite=Strict; Secure; Path=/; Max-Age=0").unwrap(),
    );
    Ok(response)
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

async fn setup(
    State(state): State<AppState>,
    Json(req): Json<SetupRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.token != state.setup_token {
        return Err(AppError::Unauthorized);
    }

    let user = state.auth.create_admin_user(&req.email, &req.password).await?;

    Ok(Json(serde_json::json!({
        "message": "Admin user created successfully",
        "user": UserResponse::from(user)
    })))
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
