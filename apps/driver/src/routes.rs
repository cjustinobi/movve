use axum::{
    routing::get,
    Router,
};
use crate::{handlers, docs, AppState};
use utoipa::OpenApi;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/driver/health", get(handlers::health_check))
        .route("/api/driver/drivers", get(handlers::list_drivers).post(handlers::create_driver))
        .route("/api/driver/drivers/{id}", get(handlers::get_driver))
        .route("/openapi.json", get(|| async {
            axum::Json(docs::DriverApiDoc::openapi())
        }))
}