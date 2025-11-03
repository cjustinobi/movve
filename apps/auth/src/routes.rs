use axum::{
    routing::{get, post},
    Router,
};
use crate::{handlers, docs, AppState};
use utoipa::OpenApi;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/health", get(health_check))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/verify", get(handlers::verify_token))
        .route("/api/auth/forgot-password", post(handlers::forgot_password))
        // Expose OpenAPI spec as JSON endpoint for gateway to fetch
        .route("/openapi.json", get(|| async {
            axum::Json(docs::AuthApiDoc::openapi())
        }))
}

async fn health_check() -> &'static str {
    "Auth service is healthy"
}