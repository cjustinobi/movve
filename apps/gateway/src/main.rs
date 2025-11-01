mod proxy;
mod docs;

use axum::{
    middleware,
    routing::{any, get, post},
    Router,
};
use common::{middleware::jwt_auth, AppConfig};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
    let http_client = reqwest::Client::new();

    let app_state = AppState {
        config: config.clone(),
        http_client,
    };

    let config_for_middleware = config.clone();

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(proxy::proxy_to_auth))
        .route("/api/auth/login", post(proxy::proxy_to_auth));

    // Protected routes (require JWT authentication)
    let protected_routes = Router::new()
        .route("/api/auth/verify", get(proxy::proxy_to_auth))
        .route("/api/drivers", any(proxy::proxy_to_driver))
        .route("/api/drivers/{id}", any(proxy::proxy_to_driver))
        .layer(middleware::from_fn(move |req, next| {
            let secret = config_for_middleware.jwt.secret.clone();
            jwt_auth(secret, req, next)
        }));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // Serve the dynamically merged OpenAPI spec at a custom path
        .route("/merged-openapi.json", get(docs::get_merged_openapi))
        // Swagger UI points to our custom merged spec
        .merge(
            SwaggerUi::new("/docs")
                .url("/merged-openapi.json", docs::GatewayApiDoc::openapi())
        )
        .with_state(app_state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("🚪 Gateway listening on {}", addr);
    tracing::info!("📚 Swagger UI available at: http://{}/docs", addr);
    tracing::info!("📡 Auth service: {}", config.services.auth_service_url);
    tracing::info!("🚗 Driver service: {}", config.services.driver_service_url);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Gateway is healthy"
}