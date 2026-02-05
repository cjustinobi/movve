use axum::serve;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

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

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let state = AppState { pool };

    let app = routes::create_routes(state);

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8004".to_string());
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    info!("Admin service listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}
