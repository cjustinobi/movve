mod handlers;
mod repository;
mod service;
mod model;
mod docs;
mod schema;

use axum::{
    routing::{get, post},
    Router,
};
use common::AppConfig;
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use repository::UserRepository;
use service::AuthService;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;

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

    // Create Diesel connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(5)
        .build(manager)
        .expect("Failed to create pool");

    // Run migrations (if you have diesel_migrations)
    // You'll need to add: diesel_migrations = "2.2.0" to Cargo.toml
    // Uncomment the following if using diesel_migrations:
    /*
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
    
    let mut conn = pool.get().expect("Failed to get connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    */

    let user_repo = UserRepository::new(pool);
    let auth_service = Arc::new(AuthService::new(user_repo, config.jwt.clone()));

    let app_state = AppState { auth_service };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/verify", get(handlers::verify_token))
        .route("/api/auth/forgot-password", post(handlers::forgot_password))
        // Expose OpenAPI spec as JSON endpoint for gateway to fetch
        .route("/openapi.json", get(|| async {
            axum::Json(docs::AuthApiDoc::openapi())
        }))
        .with_state(app_state);

    let addr: String = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🔐 Auth service listening on {} (localhost only)", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Auth service is healthy"
}