mod handlers;
mod repository;
mod service;
mod model;
mod docs;
mod schema;
mod database;
mod routes;

use common::AppConfig;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use services::MailService;
use std::sync::Arc;
use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use repository::DriverRepository;
use service::DriverService;

#[derive(Clone)]
pub struct AppState {
    pub driver_service: Arc<DriverService>,
    pub mail_service: Arc<MailService>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration (from libs/common)
    let config = AppConfig::load()?;
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Setup Diesel connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool");

    // Initialize repository and service layer
    let repo = DriverRepository::new(pool);
    let service = Arc::new(DriverService::new(repo, config.jwt.clone()));

    let mail_service = Arc::new(MailService::new(
        config.mail.api_key.clone(),
        config.mail.from_email.clone(),
    ));
    // Shared app state
    let state = AppState {
        driver_service: service,
        mail_service,
    };

    // Use the routes module
    let app = routes::create_routes(state);

    let addr = format!("{}:{}", config.services.driver_service_host, config.services.driver_service_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚗 Driver service listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}