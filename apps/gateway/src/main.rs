mod proxy;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use common::{middleware::jwt_auth, AppConfig};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub http_client: reqwest::Client,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(AppConfig::load()?);
    let http_client = reqwest::Client::new();

    let app_state = AppState {
        config: config.clone(),
        http_client,
    };

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(proxy::proxy_to_auth))
        .route("/api/auth/login", post(proxy::proxy_to_auth));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/api/auth/verify", get(proxy::proxy_to_auth))
        .route("/api/driver/*path", get(proxy::proxy_to_driver).post(proxy::proxy_to_driver))
        .layer(middleware::from_fn_with_state(
            config.jwt.clone(),
            jwt_auth,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(app_state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Gateway listening on {}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Gateway is healthy"
}