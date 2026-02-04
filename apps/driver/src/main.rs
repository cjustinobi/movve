mod database;
mod docs;
mod handlers;
mod model;
mod repository;
mod routes;
mod schema;
mod service;

use common::AppConfig;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy::dotenv;
use services::{AuthServiceClient, CloudinaryService, MailService, RiderServiceClient};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use repository::DriverRepository;
use service::DriverService;

#[derive(Clone)]
pub struct AppState {
    pub driver_service: Arc<DriverService>,
    pub mail_service: Arc<MailService>,
    pub auth_service: Arc<AuthServiceClient>,
    pub cloudinary_service: Arc<CloudinaryService>,
    pub rider_service: Arc<RiderServiceClient>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration (from libs/common)
    let config = AppConfig::load()?;
    let database_url =
        std::env::var("DRIVER_DATABASE_URL").expect("DRIVER_DATABASE_URL must be set");

    // Setup Diesel connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool");

    // Initialize repository and service layer
    let auth_service = Arc::new(AuthServiceClient::new(
        config.services.auth_service_url.clone(),
    ));
    let repo = DriverRepository::new(pool);
    let service = Arc::new(DriverService::new(
        repo,
        config.jwt.clone(),
        auth_service.clone(),
    ));

    let mail_service = Arc::new(MailService::new(
        config.mail.api_key.clone(),
        config.mail.from_email.clone(),
    ));

    let cloudinary_service = Arc::new(CloudinaryService::new(&config));

    let rider_service = Arc::new(RiderServiceClient::new(
        config.services.rider_service_url.clone(),
    ));

    // Shared app state
    let state = AppState {
        driver_service: service,
        mail_service,
        auth_service,
        cloudinary_service,
        rider_service,
    };

    // Use the routes module
    let app = routes::create_routes(state);

    let addr = format!(
        "{}:{}",
        config.services.driver_service_host, config.services.driver_service_port
    );
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚗 Driver service listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
