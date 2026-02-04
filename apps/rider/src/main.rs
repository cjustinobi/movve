mod chat_service;
mod database;
mod distance_service;
mod docs;
mod driver_client;
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
use services::MailService;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use chat_service::ChatService;
use distance_service::DistanceService;
use driver_client::DriverClient;
use repository::RiderRepository;
use service::RiderService;

pub struct AppState {
    pub rider_service: Arc<RiderService>,
    pub chat_service: Arc<ChatService>,
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
    let database_url = std::env::var("RIDER_DATABASE_URL").expect("RIDER_DATABASE_URL must be set");

    // Setup Diesel connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool");

    // Initialize repository and service layer
    let repo = RiderRepository::new(pool);

    let driver_service_url = config.services.driver_service_url.clone();
    let driver_client = Arc::new(DriverClient::new(driver_service_url));

    // Initialize distance service with Google Maps API key (optional)
    let google_api_key = std::env::var("GOOGLE_MAPS_API_KEY").ok();
    if google_api_key.is_none() {
        tracing::warn!(
            "GOOGLE_MAPS_API_KEY not set, will use Haversine formula for distance calculation"
        );
    }
    let distance_service = Arc::new(DistanceService::new(google_api_key));

    // Initialize rider service with all dependencies
    let service = Arc::new(RiderService::new(
        repo.clone(),
        config.jwt.clone(),
        driver_client,
        distance_service,
    ));

    let mail_service = Arc::new(MailService::new(
        config.mail.api_key.clone(),
        config.mail.from_email.clone(),
    ));

    // Initialize chat service
    let chat_service = Arc::new(ChatService::new(repo.clone()));

    // Shared app state
    let state = AppState {
        rider_service: service,
        chat_service,
        mail_service,
    };

    // Use the routes module
    let app = routes::create_routes(state);

    let addr = format!(
        "{}:{}",
        config.services.rider_service_host, config.services.rider_service_port
    );
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚗 Rider service listening on {}", addr);
    tracing::info!(
        "📍 Distance calculation: {}",
        if std::env::var("GOOGLE_MAPS_API_KEY").is_ok() {
            "Google Maps API"
        } else {
            "Haversine formula (fallback)"
        }
    );

    axum::serve(listener, app).await?;
    Ok(())
}
