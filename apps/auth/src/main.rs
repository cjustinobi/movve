mod handlers;
mod repository;
mod service;
mod model;
mod docs;
mod schema;
mod routes;

use common::AppConfig;
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use repository::UserRepository;
use service::AuthService;
use services::MailService;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub mail_service: Arc<MailService>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::load()?;
    let database_url = std::env::var("AUTH_DATABASE_URL")
        .expect("AUTH_DATABASE_URL must be set");

    // Create Diesel connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(5)
        .build(manager)
        .expect("Failed to create pool");

    let user_repo = UserRepository::new(pool);
    let auth_service = Arc::new(AuthService::new(user_repo, config.jwt.clone()));

    let mail_service = Arc::new(MailService::new(
        config.mail.api_key.clone(),
        config.mail.from_email.clone(),
    ));

    let app_state = AppState { 
        auth_service,
        mail_service
     };

    // Use the routes module
    let app = routes::create_routes(app_state);

    let addr = format!("{}:{}", config.services.auth_service_host, config.services.auth_service_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🔐 Auth service listening on {} (localhost only)", addr);

    axum::serve(listener, app).await?;

    Ok(())
}