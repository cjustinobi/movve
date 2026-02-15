use crate::{AppState, docs, handlers};
use axum::{
    Router,
    routing::{delete, get, post, put},
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
        // User Management
        .route("/api/admin/users", get(handlers::get_users))
        .route("/api/admin/stats/users", get(handlers::get_user_stats))
        .route(
            "/api/admin/users/{id}",
            delete(handlers::delete_user).put(handlers::update_user),
        )
        .route(
            "/api/admin/users/{id}/suspend",
            post(handlers::toggle_user_suspension),
        )
        .route(
            "/api/admin/users/{id}/details",
            get(handlers::get_user_details),
        )
        // Driver Management
        .route(
            "/api/driver/admin/drivers/locations",
            get(handlers::get_online_driver_locations),
        )
        .route(
            "/api/driver/admin/stats/drivers",
            get(handlers::get_driver_stats),
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
