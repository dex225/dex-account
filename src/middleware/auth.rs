use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::error::AppError;
use crate::services::CryptoService;

#[derive(Clone)]
pub struct AuthState {
    pub crypto: Arc<CryptoService>,
}

pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(AppError::Unauthorized),
    };

    let claims = state.crypto.validate_token(token)?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    req.extensions_mut().insert(UserId(user_id));

    Ok(next.run(req).await)
}

#[derive(Clone, Copy)]
pub struct UserId(pub Uuid);

#[derive(Clone, Copy)]
pub struct UserRole(pub &'static str);
