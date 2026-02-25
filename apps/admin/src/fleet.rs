use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use common::{ApiResponse, AppError};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::AppState;

// Admin-specific vehicle types schema mapping to the driver database
diesel::table! {
    vehicle_types (id) {
        id -> Uuid,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 50]
        display_name -> Varchar,
        description -> Text,
        base_price -> Float8,
        is_active -> Bool,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

// Model for Vehicle Types
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, ToSchema)]
#[diesel(table_name = vehicle_types)]
pub struct VehicleTypeModel {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub base_price: f64,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// DTO for update
#[derive(Deserialize, ToSchema, AsChangeset)]
#[diesel(table_name = vehicle_types)]
pub struct UpdateVehicleTypeRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub base_price: f64,
    pub is_active: Option<bool>,
}

/// Lists all vehicle types
#[utoipa::path(
    get,
    path = "/api/admin/vehicle-types",
    responses(
        (status = 200, description = "List of vehicle types", body = ApiResponse<Vec<VehicleTypeModel>>),
    ),
    tag = "Admin Fleet",
    security(("bearerAuth" = []))
)]
pub async fn get_vehicle_types(
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<VehicleTypeModel>>, AppError> {
    let pool = state.driver_pool.clone();

    let types = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        vehicle_types::dsl::vehicle_types
            .order(vehicle_types::dsl::name.asc())
            .select(VehicleTypeModel::as_select())
            .load::<VehicleTypeModel>(&mut conn)
            .map_err(|e| AppError::InternalError(e.to_string()))
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::success(types))
}

/// Updates a vehicle type
#[utoipa::path(
    put,
    path = "/api/admin/vehicle-types/{id}",
    request_body = UpdateVehicleTypeRequest,
    params(
        ("id" = Uuid, Path, description = "Vehicle Type ID")
    ),
    responses(
        (status = 200, description = "Vehicle type updated", body = ApiResponse<VehicleTypeModel>),
        (status = 404, description = "Vehicle type not found"),
    ),
    tag = "Admin Fleet",
    security(("bearerAuth" = []))
)]
pub async fn update_vehicle_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateVehicleTypeRequest>,
) -> Result<ApiResponse<VehicleTypeModel>, AppError> {
    let pool = state.driver_pool.clone();

    let updated = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        diesel::update(vehicle_types::dsl::vehicle_types.filter(vehicle_types::dsl::id.eq(id)))
            .set(&req)
            .returning(VehicleTypeModel::as_returning())
            .get_result::<VehicleTypeModel>(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound("Vehicle type not found".to_string())
                }
                _ => AppError::InternalError(e.to_string()),
            })
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))??;

    Ok(ApiResponse::success_with_message(
        "Vehicle type updated",
        updated,
    ))
}
