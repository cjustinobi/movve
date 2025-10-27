use axum::{routing::get, Json, Router};
use tracing_subscriber;
use tokio::net::TcpListener;
use common::Driver;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();


	let app = Router::new()
	.route("/health", get(health))
	.route("/drivers", get(list_drivers));

	let listener = TcpListener::bind("0.0.0.0:4002").await?;
	tracing::info!("driver listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, app)
	.await?;
	Ok(())
}


async fn health() -> &'static str {
"ok"
}


async fn list_drivers() -> Json<Vec<Driver>> {
let drivers = vec![
Driver { id: 1, name: "Alice".into() },
Driver { id: 2, name: "Bob".into() },
];
Json(drivers)
}