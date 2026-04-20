use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::error::AppError;

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(AppError::Unauthorized),
    };

    let claims = req
        .extensions()
        .get::<crate::models::Claims>()
        .cloned();

    if claims.is_none() {
        return Err(AppError::Unauthorized);
    }

    let user_id = Uuid::parse_str(&claims.as_ref().unwrap().sub)
        .map_err(|_| AppError::InvalidToken)?;

    req.extensions_mut().insert(UserId(user_id));

    Ok(next.run(req).await)
}

#[derive(Clone, Copy)]
pub struct UserId(pub Uuid);

#[derive(Clone, Copy)]
pub struct UserRole(pub &'static str);
