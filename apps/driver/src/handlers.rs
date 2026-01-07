use axum::{extract::{State, Path}, Json};
use common::{ApiResponse, AppError};
use uuid::Uuid;
use crate::{
    AppState,
    model::{Driver, NewDriver}};
use serde_json::json;

#[utoipa::path(
    post,
    path = "/api/driver/drivers",
    responses(
        (status = 200, description = "Create a driver", body = [Driver])
    ),
    tag = "Driver"
)]

pub async fn create_driver(
    State(state): State<AppState>,
    Json(req): Json<NewDriver>,
) -> Result<Json<Driver>, axum::http::StatusCode> {
    match state.driver_service.register_driver(req) {
        Ok(driver) => Ok(Json(driver)),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/api/driver/drivers",
    responses(
        (status = 200, description = "List all drivers", body = [Driver])
    ),
    tag = "Driver"
)]

pub async fn list_drivers(
    State(state): State<AppState>,
) -> Result<Json<Vec<Driver>>, axum::http::StatusCode> {
    match state.driver_service.list_drivers() {
        Ok(drivers) => Ok(Json(drivers)),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/api/driver/drivers/{id}",
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Driver retrieved successfully", body = Driver),
        (status = 404, description = "Driver not found")
    ),
    tag = "Driver"
)]
pub async fn get_driver(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
) -> Result<Json<Driver>, axum::http::StatusCode> {
    match state.driver_service.get_driver(driver_id) {
        Ok(driver) => Ok(Json(driver)),
        Err(_) => Err(axum::http::StatusCode::NOT_FOUND),
    }
}



pub async fn notify_driver(
    State(state): State<AppState>,
    Json(req): Json<()>,
) -> Result<ApiResponse<()>, AppError> {
    // Send notification email to driver
    state.mail_service.send_notification(
        "cjustinobi@gmail.com",
        "New Ride Request",
        "<h1>You have a new ride request</h1><p>Check your app for details.</p>",
        Some("You have a new ride request. Check your app for details."),
    ).await;
    
    Ok(ApiResponse::success_with_message("Notification sent", ()))
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({"status": "Driver service is healthy"}))
}
