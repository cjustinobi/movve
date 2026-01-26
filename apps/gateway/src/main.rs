mod proxy;
mod docs;

use axum::{
    middleware,
    routing::{any, get, post},
    Router,
};
use common::{middleware as app_middleware, AppConfig};
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


    pub fn public_routes() -> Router<AppState> {
        Router::new()
            .route("/health", get(health_check))
            .route("/", get(root_handler))
}

    /// Documentation routes (Swagger UI and OpenAPI spec)
    pub fn docs_routes() -> Router<AppState> {
        Router::new()
            .route("/api-docs/openapi.json", get(docs::get_merged_openapi))
            .merge(
                SwaggerUi::new("/docs")
                    .config(Config::new(["/api-docs/openapi.json"]))
            )
    }

/// Auth service routes - mix of public and protected
pub fn auth_routes(jwt_secret: String) -> Router<AppState> {
    // Public auth endpoints (no authentication required)
    let public = Router::new()
        .route("/api/auth/register", post(proxy::proxy_by_prefix))
        .route("/api/auth/login", post(proxy::proxy_by_prefix))
        .route("/api/auth/forgot-password", post(proxy::proxy_by_prefix))
        .route("/api/auth/reset-password", post(proxy::proxy_by_prefix))
        .route("/api/auth/resend-verification", post(proxy::proxy_by_prefix));

    // Protected auth endpoints (require JWT)
    let protected = Router::new()
        .route("/api/auth/verify", get(proxy::proxy_by_prefix))
        .route("/api/auth/{*path}", any(proxy::proxy_by_prefix))
        .route_layer(middleware::from_fn(move |req, next| {
            let secret = jwt_secret.clone();
            async move {
                app_middleware::jwt_auth_middleware(secret, req, next).await
            }
        }));

    public.merge(protected)
}

/// Protected service routes (all require JWT authentication)
pub fn protected_routes(jwt_secret: String) -> Router<AppState> {
    Router::new()
        .route("/api/driver/{*path}", any(proxy::proxy_by_prefix))
        .route("/api/rider/{*path}", any(proxy::proxy_by_prefix))
        .route("/api/trip/{*path}", any(proxy::proxy_by_prefix))
        .route_layer(middleware::from_fn(move |req, next| {
            let secret = jwt_secret.clone();
            async move {
                app_middleware::jwt_auth_middleware(secret, req, next).await
            }
        }))
}

    let app = Router::new()
        .merge(public_routes())
        .merge(auth_routes(config.jwt.secret.clone()))
        .merge(protected_routes(config.jwt.secret.clone()))
        .merge(docs_routes())
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
    "Gateway is healthy."
}
async fn root_handler() -> &'static str {
    "Movve API Gateway - Visit /docs for API documentation"
}

