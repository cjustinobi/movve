use axum::serve;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::env;
use common::AppConfig;

mod docs;
mod handlers;
mod model;
mod routes;
mod schema;

#[derive(Clone)]
pub struct AppState {
    pub pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let config = AppConfig::load()?;
    let database_url = env::var("ADMIN_DATABASE_URL").expect("ADMIN_DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let state = AppState { pool };

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
