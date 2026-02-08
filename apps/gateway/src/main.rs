mod docs;
mod proxy;

use axum::{
    Router,
    routing::{any, get},
};
use common::{AppConfig};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa_swagger_ui::{Config, SwaggerUi};

impl AsRef<AppConfig> for AppState {
    fn as_ref(&self) -> &AppConfig {
        &self.config
    }
}

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
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let app_state = AppState {
        config: config.clone(),
        http_client,
    };

    pub fn public_routes() -> Router<AppState> {
        Router::new()
            .route("/health", get(health_check))
            .route("/", get(root_handler))
    }

    /// Documentation routes (Swagger UI and OpenAPI spec)
    pub fn docs_routes() -> Router<AppState> {
        Router::new()
            .route("/api-docs/openapi.json", get(docs::get_merged_openapi))
            .merge(SwaggerUi::new("/docs").config(Config::new(["/api-docs/openapi.json"])))
    }

    /// API service routes - proxy to microservices and let them handle their own security
    pub fn api_routes() -> Router<AppState> {
        Router::new()
            .route("/api/auth/{*path}", any(proxy::proxy_by_prefix))
            .route("/api/driver/{*path}", any(proxy::proxy_by_prefix))
            .route("/api/rider/{*path}", any(proxy::proxy_by_prefix))
            .route("/api/trip/{*path}", any(proxy::proxy_by_prefix))
            .route("/api/admin/{*path}", any(proxy::proxy_by_prefix))
    }

    let app = Router::new()
        .merge(public_routes())
        .merge(api_routes())
        .merge(docs_routes())
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit
        .with_state(app_state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| config.server.port.to_string())
        .parse::<u16>()?;

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚪 Gateway listening on {}", addr);
    tracing::info!("📚 Swagger UI: http://{}/docs", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Gateway is healthy"
}
async fn root_handler() -> &'static str {
    "Movve API Gateway - Visit /docs for API documentation"
}
