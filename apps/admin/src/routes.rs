use crate::{handlers, AppState};
use axum::{routing::put, Router};

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
        .with_state(state)
}
