use axum::serve;
use common::AppConfig;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::env;

mod docs;
mod fleet;
mod handlers;
mod model;
mod routes;
mod schema;

#[derive(Clone)]
pub struct AppState {
    pub auth_pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    pub driver_pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let config = AppConfig::load()?;

    // Auth DB Pool
    let auth_db_url = env::var("AUTH_DATABASE_URL").expect("AUTH_DATABASE_URL must be set");
    let auth_manager = ConnectionManager::<PgConnection>::new(auth_db_url);
    let auth_pool = r2d2::Pool::builder()
        .build(auth_manager)
        .expect("Failed to create auth pool.");

    // Driver DB Pool
    let driver_db_url = env::var("DRIVER_DATABASE_URL").expect("DRIVER_DATABASE_URL must be set");
    let driver_manager = ConnectionManager::<PgConnection>::new(driver_db_url);
    let driver_pool = r2d2::Pool::builder()
        .build(driver_manager)
        .expect("Failed to create driver pool.");

    let state = AppState {
        auth_pool,
        driver_pool,
    };

    let app = routes::create_routes(state);

    let addr = format!(
        "{}:{}",
        config.services.admin_service_host, config.services.admin_service_port
    );
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Admin service listening on {}", addr);
    serve(listener, app).await?;

    Ok(())
}
