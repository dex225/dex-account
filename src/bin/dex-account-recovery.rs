use std::env;

use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Header, EncodingKey};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let admin_email = env::var("DEX_ADMIN_EMAIL").expect("DEX_ADMIN_EMAIL must be set");
    let jwt_secret = env::var("DEX_JWT_SECRET").expect("DEX_JWT_SECRET must be set");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let user: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&admin_email)
        .fetch_optional(&pool)
        .await?;

    let user_id = match user {
        Some((id,)) => id,
        None => {
            eprintln!("User with email {} not found", admin_email);
            std::process::exit(1);
        }
    };

    let now = Utc::now();
    let exp = now + Duration::minutes(15);

    let claims = serde_json::json!({
        "sub": user_id.to_string(),
        "role": "Admin",
        "exp": exp.timestamp(),
        "iat": now.timestamp()
    });

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;

    println!("Emergency recovery token for {}:", admin_email);
    println!("{}", token);
    println!("\nToken expires in 15 minutes.");
    println!("Use this token as Bearer token in Authorization header.");

    Ok(())
}
