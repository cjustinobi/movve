use axum::{routing::{get, post}, Json, Router};
use tracing_subscriber;
use tokio::net::TcpListener;
use common::{LoginRequest, LoginResponse};


#[tokio::main]
async fn main() -> anyhow::Result<()>  {
    tracing_subscriber::fmt::init();

    let app = Router::new()
    .route("/health", get(health))
    .route("/login", post(login));


    let listener = TcpListener::bind("0.0.0.0:4001").await.unwrap();
    tracing::info!("auth listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
    .await?;
    Ok(())
}


async fn health() -> &'static str {
"ok"
}


async fn login(Json(payload): Json<LoginRequest>) -> Json<LoginResponse> {
tracing::info!("login attempt: {}", payload.username);
// NOTE: this is a demo — do NOT use in production. Use proper auth + hashing + token systems.
let resp = LoginResponse { token: format!("token-for-{}", payload.username) };
Json(resp)
}