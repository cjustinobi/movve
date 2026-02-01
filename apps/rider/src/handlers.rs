use axum::{Extension, Json, extract::{State, Path}, http::StatusCode};
use common::{ApiResponse, AppError, EmptyData};
use tracing::info;
use uuid::Uuid;
use crate::{
    AppState,
    model::{
        Claims, CreateRideRequest, Location, PayRideRequest, RateDriverRequest, RideEstimateRequest, RideEstimateResponse, RideResponse
    },
};

/// Ride Estimate
#[utoipa::path(
    post,
    path = "/api/rider/preview",
    request_body = RideEstimateRequest,
    responses(
        (status = 200, description = "Ride estimated successfully", body = ApiResponse<RideEstimateResponse>),
    ),
    tag = "Rider",
    security(("bearerAuth" = []))
)]
pub async fn estimate_ride(
    State(state): State<AppState>,
    Json(req): Json<RideEstimateRequest>,
) -> Result<ApiResponse<RideEstimateResponse>, AppError> {
    info!("Estimating ride: pickup=({}, {}), destination=({}, {}), vehicle_type={}",
        req.pickup.latitude, req.pickup.longitude,
        req.destination.latitude, req.destination.longitude,
        req.vehicle_type);
    let response = state.rider_service.estimate_ride(req).await?;
    Ok(ApiResponse::success(response))
}

/// Create Ride
#[utoipa::path(
    post,
    path = "/api/rider/rides",
    request_body = CreateRideRequest,
    responses(
        (status = 201, description = "Ride created successfully", body = ApiResponse<RideResponse>),
    ),
    tag = "Rider",
    security(("bearerAuth" = []))
)]
pub async fn create_ride(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRideRequest>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    info!("Creating ride request: pickup={}, destination={}, driver_id={}", 
        req.pickup.address, req.destination.address, req.driver_id);
    
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;
    
    info!("User ID from token: {}", user_id);

    let response = state.rider_service.create_ride(user_id, req).await?;
    Ok(ApiResponse::success_with_message("Ride requested successfully", response))
}

/// Get Ride
#[utoipa::path(
    get,
    path = "/api/rider/{id}",
    responses(
        (status = 200, description = "Ride details", body = ApiResponse<RideResponse>),
        (status = 404, description = "Ride not found"),
    ),
    params(
        ("id" = Uuid, Path, description = "Ride ID")
    ),
    tag = "Rider",
    security(("bearerAuth" = []))
)]
pub async fn get_ride(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    let response = state.rider_service.get_ride(id).await?;
    Ok(ApiResponse::success(response))
}

/// Get Ride History
#[utoipa::path(
    get,
    path = "/api/rider/rides",
    responses(
        (status = 200, description = "Ride history", body = ApiResponse<Vec<RideResponse>>),
    ),
    tag = "Rider",
    security(("bearerAuth" = []))
)]
pub async fn get_rides(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<ApiResponse<Vec<RideResponse>>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let response = state.rider_service.get_rider_history(user_id).await?;
    Ok(ApiResponse::success(response))
}

/// Cancel Ride
#[utoipa::path(
    post,
    path = "/api/rider/{id}/cancel",
    responses(
        (status = 200, description = "Ride cancelled", body = ApiResponse<RideResponse>),
    ),
    params(
        ("id" = Uuid, Path, description = "Ride ID")
    ),
    tag = "Rider",
    security(("bearerAuth" = []))
)]
pub async fn cancel_ride(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let response = state.rider_service.cancel_ride(id, user_id).await?;
    Ok(ApiResponse::success_with_message("Ride cancelled", response))
}

/// Pay Ride
#[utoipa::path(
    post,
    path = "/api/rider/{id}/pay",
    request_body = PayRideRequest,
    responses(
        (status = 200, description = "Ride paid", body = ApiResponse<RideResponse>),
    ),
    params(
        ("id" = Uuid, Path, description = "Ride ID")
    ),
    tag = "Rider",
    security(("bearerAuth" = []))
)]
pub async fn pay_ride(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<PayRideRequest>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let response = state.rider_service.pay_ride(id, user_id, req).await?;
    Ok(ApiResponse::success_with_message("Payment successful", response))
}

/// Rate Driver
#[utoipa::path(
    post,
    path = "/api/rider/{id}/rate",
    request_body = RateDriverRequest,
    responses(
        (status = 200, description = "Driver rated", body = ApiResponse<EmptyData>),
    ),
    params(
        ("id" = Uuid, Path, description = "Ride ID")
    ),
    tag = "Rider",
    security(("bearerAuth" = []))
)]
pub async fn rate_driver(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<RateDriverRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    state.rider_service.rate_driver(id, user_id, req).await?;
    Ok(ApiResponse::message_only(StatusCode::OK, "Rating submitted thank you"))
}

/// Driver Location
#[utoipa::path(
    get,
    path = "/api/rider/{id}/driver-location",
    responses(
        (status = 200, description = "Driver location", body = ApiResponse<Location>),
    ),
    params(
        ("id" = Uuid, Path, description = "Ride ID")
    ),
    tag = "Rider",
    security(("bearerAuth" = []))
)]
pub async fn get_driver_location(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<Location>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let location = state.rider_service.get_driver_location(id, user_id).await?;
    Ok(ApiResponse::success(location))
}

/// Ride Status
#[utoipa::path(
    get,
    path = "/api/rider/{id}/status",
    responses(
        (status = 200, description = "Ride status"),
    ),
    params(
        ("id" = Uuid, Path, description = "Ride ID")
    ),
    tag = "Rider",
    security(("bearerAuth" = []))
)]
pub async fn get_ride_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    let ride = state.rider_service.get_ride(id).await?;
    Ok(ApiResponse::success(serde_json::json!({
        "status": ride.status
    })))
}
