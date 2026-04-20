mod db;
mod error;
mod middleware;
mod models;
mod routes;
mod services;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Router,
};
use tokio::time::interval;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::db::{create_pool, DbPool};
use crate::routes::create_router;
use crate::services::{AuthService, CryptoService};

#[derive(Clone)]
struct AppState {
    pool: DbPool,
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn ready(State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    sqlx::query("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dex_account=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("DEX_JWT_SECRET").expect("DEX_JWT_SECRET must be set");
    let emergency_api_key =
        std::env::var("DEX_EMERGENCY_API_KEY").expect("DEX_EMERGENCY_API_KEY must be set");

    let pool = create_pool(&database_url).await?;

    let crypto = Arc::new(CryptoService::new(jwt_secret));
    let auth = Arc::new(AuthService::new(pool.clone(), crypto.clone()));

    let app_state = AppState { pool };

    let cors = CorsLayer::new()
        .allow_origin([
            "https://myaccount.dex.com.br".parse().unwrap(),
            "https://app.dex.com.br".parse().unwrap(),
        ])
        .allow_credentials(true)
        .allow_methods(Any)
        .allow_headers(Any);

    let auth_router = create_router(auth.clone(), crypto.clone(), emergency_api_key);

    let health_router = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .with_state(app_state.clone());

    let app = Router::new()
        .nest("/api/v1", auth_router)
        .merge(health_router)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let auth_for_cleanup = auth.clone();
    tokio::spawn(async move {
        let mut timer = interval(Duration::from_secs(3600));
        loop {
            timer.tick().await;
            if let Err(e) = auth_for_cleanup.cleanup_expired_tokens().await {
                tracing::error!("Cleanup failed: {:?}", e);
            }
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server starting on 0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
