use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use common::{ApiResponse, AppError, EmptyData};
use diesel::prelude::*;
use tracing::info;
use uuid::Uuid;

use crate::{model::Driver, schema::drivers::dsl::*, AppState};

#[utoipa::path(
    put,
    path = "/api/admin/drivers/{id}/approve-license",
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Driver license valid outcome", body = ApiResponse<EmptyData>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Admin"
)]
pub async fn approve_driver_license_image(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    
    let mut conn = state
        .pool
        .get()
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    diesel::update(drivers.filter(id.eq(driver_id)))
        .set(driver_license_verified.eq(true))
        .execute(&mut conn)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Driver license image approved",
    ))
}

#[utoipa::path(
    put,
    path = "/api/admin/drivers/{id}/approve-insurance",
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Insurance valid outcome", body = ApiResponse<EmptyData>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Admin"
)]
pub async fn approve_insurance_image(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
) -> Result<ApiResponse<EmptyData>, AppError> {

    let mut conn = state
        .pool
        .get()
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    diesel::update(drivers.filter(id.eq(driver_id)))
        .set(insurance_verified.eq(true))
        .execute(&mut conn)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Insurance image approved",
    ))
}

#[utoipa::path(
    put,
    path = "/api/admin/drivers/{id}/approve-vehicle-image",
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Vehicle image valid outcome", body = ApiResponse<EmptyData>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Admin"
)]
pub async fn approve_vehicle_image(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
) -> Result<ApiResponse<EmptyData>, AppError> {

    let mut conn = state
        .pool
        .get()
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    diesel::update(drivers.filter(id.eq(driver_id)))
        .set(vehicle_image_verified.eq(true))
        .execute(&mut conn)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Vehicle image approved",
    ))
}
