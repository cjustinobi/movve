use crate::{
    AppState,
    model::{Driver, NewDriver},
};
use axum::{
    Json,
    extract::{Path, State},
};
use common::{ApiResponse, AppError};
use serde_json::json;
use tracing::info;
use uuid::Uuid;

/// Creates a new driver
///
/// Creates a new driver with the provided details.
#[utoipa::path(
    post,
    path = "/api/driver/drivers",
    responses(
        (status = 200, description = "Create a driver", body = ApiResponse<Driver>)
    ),
    tag = "Driver",
    security(("bearerAuth" = []))
)]

pub async fn create_driver(
    State(state): State<AppState>,
    Json(req): Json<NewDriver>,
) -> Result<ApiResponse<Driver>, AppError> {
    info!("Creating new driver: {:?}", req);
    let response = state
        .driver_service
        .create_driver(req)
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(ApiResponse::success_with_message(
        "Driver created successfully",
        response,
    ))
}

#[utoipa::path(
    get,
    path = "/api/driver/drivers",
    responses(
        (status = 200, description = "List all drivers", body = ApiResponse<Vec<Driver>>)
    ),
    tag = "Driver"
)]

pub async fn list_drivers(
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<Driver>>, AppError> {
    let drivers = state
        .driver_service
        .list_drivers()
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ApiResponse::success(drivers))
}

#[utoipa::path(
    get,
    path = "/api/driver/drivers/{id}",
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Driver retrieved successfully", body = ApiResponse<Driver>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Driver"
)]
pub async fn get_driver(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
) -> Result<ApiResponse<Driver>, AppError> {
    let driver = state
        .driver_service
        .get_driver(driver_id)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ApiResponse::success(driver))
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({"status": "Driver service is healthy"}))
}
