mod handlers;
mod repository;
mod service;

use axum::{
    routing::{get, post},
    Router,
};
use common::AppConfig;
use repository::UserRepository;
use service::AuthService;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::load()?;
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let user_repo = UserRepository::new(pool);
    let auth_service = Arc::new(AuthService::new(user_repo, config.jwt.clone()));

    let app_state = AppState { auth_service };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/verify", get(handlers::verify_token))
        .with_state(app_state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Auth service listening on {}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Auth service is healthy"
}