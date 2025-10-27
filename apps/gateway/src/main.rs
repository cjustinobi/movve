use axum::{body::Body, http::{Request, StatusCode}, response::IntoResponse, routing::post, Router, Extension};
use tracing_subscriber;
use tokio::net::TcpListener;
use reqwest::Client;
use axum::response::Response;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
tracing_subscriber::fmt::init();
let client = Client::new();


let app = Router::new()
.route("/health", axum::routing::get(|| async { "ok" }))
// simple pass-through endpoints
.route("/api/auth/login", post(proxy_login))
.route("/api/driver/drivers", axum::routing::get(proxy_drivers))
.layer(axum::Extension(client));


let listener = TcpListener::bind("0.0.0.0:4000").await?;
tracing::info!("gateway listening on {}", listener.local_addr().unwrap());
axum::serve(listener, app)
	.await?;
	Ok(())
}


async fn proxy_login(Extension(client): Extension<Client>, body: String) -> impl IntoResponse {
// Forward the JSON body to the auth service
let auth_url = std::env::var("AUTH_URL").unwrap_or_else(|_| "http://localhost:4001/login".into());
match client.post(&auth_url).body(body).header("content-type", "application/json").send().await {
Ok(resp) => {
let status = resp.status();
let bytes = resp.bytes().await.unwrap_or_default();
(status, bytes).into_response()
}
Err(e) => {
tracing::error!("error proxying login: {}", e);
(StatusCode::BAD_GATEWAY, "auth service error").into_response()
}
}
}


async fn proxy_drivers(Extension(client): Extension<Client>) -> impl IntoResponse {
let driver_url = std::env::var("DRIVER_URL").unwrap_or_else(|_| "http://localhost:4002/drivers".into());
match client.get(&driver_url).send().await {
Ok(resp) => {
let status = resp.status();
let headers = resp.headers().clone();
let bytes = resp.bytes().await.unwrap_or_default();

let mut response = Response::builder().status(status);
for (k, v) in headers.iter() {
    response = response.header(k, v);
}
response.body(Body::from(bytes)).unwrap()

}
Err(e) => {
tracing::error!("error proxying drivers: {}", e);
(StatusCode::BAD_GATEWAY, "driver service error").into_response()
}
}
}