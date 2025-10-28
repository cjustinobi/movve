use axum::{routing::get, Json, Router};
use tracing_subscriber;
use tokio::net::TcpListener;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();


	let app = Router::new()
	.route("/health", get(health));

	let listener = TcpListener::bind("0.0.0.0:4002").await?;
	tracing::info!("driver listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, app)
	.await?;
	Ok(())
}


async fn health() -> &'static str {
"ok"
}


