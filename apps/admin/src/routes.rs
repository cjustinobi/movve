use crate::{AppState, docs, handlers};
use axum::{Router, routing::put};
use utoipa::OpenApi;

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/admin/drivers/{id}/approve-license",
            put(handlers::approve_driver_license_image),
        )
        .route(
            "/api/admin/drivers/{id}/approve-insurance",
            put(handlers::approve_insurance_image),
        )
        .route(
            "/api/admin/drivers/{id}/approve-vehicle-image",
            put(handlers::approve_vehicle_image),
        )
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/api/admin/docs")
                .url("/api/admin/openapi.json", docs::AdminApiDoc::openapi()),
        )
        .with_state(state)
}
