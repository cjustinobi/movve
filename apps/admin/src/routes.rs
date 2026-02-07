use crate::{AppState, docs, handlers};
use axum::{
    Router,
    routing::{get, put},
};
use utoipa::OpenApi;

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/admin/drivers/{id}/update-license",
            put(handlers::update_driver_license_verification),
        )
        .route(
            "/api/admin/drivers/{id}/update-insurance",
            put(handlers::update_insurance_verification),
        )
        .route(
            "/api/admin/drivers/{id}/update-vehicle-insurance",
            put(handlers::update_vehicle_verification),
        )
        .route(
            "/openapi.json",
            get(|| async { axum::Json(docs::AdminApiDoc::openapi()) }),
        )
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/api/admin/docs")
                .url("/api/admin/openapi.json", docs::AdminApiDoc::openapi()),
        )
        .with_state(state)
}
