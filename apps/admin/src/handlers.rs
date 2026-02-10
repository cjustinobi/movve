use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use common::{ApiResponse, AppError, EmptyData};
use diesel::prelude::*;
use tracing::info;
use uuid::Uuid;

use crate::{AppState, schema::drivers::dsl::*};

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateVerificationRequest {
    pub verified: bool,
}

/// Update Driver License Verification Status
#[utoipa::path(
    put,
    path = "/api/admin/drivers/{id}/update-license",
    request_body = UpdateVerificationRequest,
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Driver license status updated", body = ApiResponse<EmptyData>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Admin"
)]
pub async fn update_driver_license_verification(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
    Json(req): Json<UpdateVerificationRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    diesel::update(drivers.filter(id.eq(driver_id)))
        .set(driver_license_verified.eq(req.verified))
        .execute(&mut conn)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        if req.verified {
            "Driver license approved"
        } else {
            "Driver license rejected"
        },
    ))
}

/// Update Insurance Verification Status
#[utoipa::path(
    put,
    path = "/api/admin/drivers/{id}/update-insurance",
    request_body = UpdateVerificationRequest,
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Insurance status updated", body = ApiResponse<EmptyData>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Admin"
)]
pub async fn update_insurance_verification(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
    Json(req): Json<UpdateVerificationRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    diesel::update(drivers.filter(id.eq(driver_id)))
        .set(insurance_verified.eq(req.verified))
        .execute(&mut conn)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        if req.verified {
            "Insurance approved"
        } else {
            "Insurance rejected"
        },
    ))
}

/// Update Vehicle Verification Status
#[utoipa::path(
    put,
    path = "/api/admin/drivers/{id}/update-vehicle-insurance",
    request_body = UpdateVerificationRequest,
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Vehicle verification status updated", body = ApiResponse<EmptyData>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Admin"
)]
pub async fn update_vehicle_verification(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
    Json(req): Json<UpdateVerificationRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    diesel::update(drivers.filter(id.eq(driver_id)))
        .set(vehicle_image_verified.eq(req.verified))
        .execute(&mut conn)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        if req.verified {
            "Vehicle verification updated"
        } else {
            "Vehicle verification rejected"
        },
    ))
}
