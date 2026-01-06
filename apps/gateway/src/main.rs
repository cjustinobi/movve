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
use utoipa_swagger_ui::{SwaggerUi, Config};

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

    // -------------------------
    // Public routes (NO auth)
    // -------------------------
    let public_routes = Router::new()
        .route("/health", get(health_check));

    // -------------------------
    // Public AUTH routes
    // -------------------------
    let public_auth_routes = Router::new()
        .route("/api/auth/register", post(proxy::proxy_by_prefix))
        .route("/api/auth/login", post(proxy::proxy_by_prefix))
        .route("/api/auth/forgot-password", post(proxy::proxy_by_prefix));

    // -------------------------
    // Protected routes (JWT)
    // -------------------------
    let protected_routes = Router::new()
        // auth routes that REQUIRE token
        .route("/api/auth/verify", get(proxy::proxy_by_prefix))
        .route("/api/auth/{*path}", any(proxy::proxy_by_prefix))
        .route("/api/driver/{*path}", any(proxy::proxy_by_prefix))
        .layer(middleware::from_fn(move |req, next| {
            let secret = config_for_middleware.jwt.secret.clone();
            jwt_auth(secret, req, next)
        }));

    let app = Router::new()
        .merge(public_routes)
        .merge(public_auth_routes)
        .merge(protected_routes)
        // OpenAPI spec
        .route("/api-docs/openapi.json", get(docs::get_merged_openapi))
        // Swagger UI
        .merge(
            SwaggerUi::new("/docs").config(
                Config::new(["/api-docs/openapi.json"])
            )
        )
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