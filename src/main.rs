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
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::time::interval;
use axum::http::{HeaderName, Method};
use tower_http::cors::{AllowHeaders, AllowMethods, CorsLayer};
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
    PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], 3001))
        .build()
        .expect("Failed to build Prometheus exporter");

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

    let setup_token = std::env::var("DEX_SETUP_TOKEN").expect("DEX_SETUP_TOKEN must be set");

    let pool = create_pool(&database_url).await?;

    let crypto = Arc::new(CryptoService::new(jwt_secret));
    let auth = Arc::new(AuthService::new(pool.clone(), crypto.clone()));

    let app_state = AppState { pool };

    let allowed_origins = std::env::var("DEX_ALLOWED_ORIGINS")
        .expect("DEX_ALLOWED_ORIGINS must be set");

    let origins: Vec<_> = allowed_origins
        .split(',')
        .map(|s| s.trim().parse().expect("Invalid origin in DEX_ALLOWED_ORIGINS"))
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_methods(AllowMethods::list([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-emergency-key"),
        ]));

    let auth_router = create_router(auth.clone(), crypto.clone(), emergency_api_key, setup_token);

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
