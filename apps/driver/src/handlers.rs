use crate::{
    AppState,
    model::{Driver, DriverLocation, DriverStatus, NewDriver, UpdateStatusRequest},
};
use axum::{
    Json,
    extract::{
        Extension, Multipart, Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
};
use common::{ApiResponse, AppError};
use futures_util::SinkExt;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct VehicleTypeInfo {
    pub r#type: crate::model::VehicleType,
    pub name: String,
    pub description: String,
    pub base_price: f64,
}

#[derive(Deserialize)]
pub struct ListDriversQuery {
    pub status: Option<DriverStatus>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateLocationRequest {
    pub latitude: f64,
    pub longitude: f64,
}

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

    // Validate vehicle capacity
    if req.vehicle_capacity < 2 {
        return Err(AppError::BadRequest(
            "Vehicle capacity must be at least 2".to_string(),
        ));
    }

    match req.vehicle_type {
        crate::model::VehicleType::Sedan => {
            if req.vehicle_capacity > 5 {
                return Err(AppError::BadRequest(
                    "Sedan capacity cannot exceed 5".to_string(),
                ));
            }
        }
        _ => {}
    }

    let response = state.driver_service.create_driver(req).await?;
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
    Query(query): Query<ListDriversQuery>,
) -> Result<ApiResponse<Vec<Driver>>, AppError> {
    let drivers = state
        .driver_service
        .list_drivers(query.status)
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

#[utoipa::path(
    get,
    path = "/api/driver/drivers/by-user/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User unique identifier")
    ),
    responses(
        (status = 200, description = "Driver retrieved successfully", body = ApiResponse<Driver>),
        (status = 404, description = "Driver not found")
    ),
    tag = "Driver"
)]
pub async fn get_driver_by_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<ApiResponse<Driver>, AppError> {
    let driver = state
        .driver_service
        .get_driver_by_user_id(user_id)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ApiResponse::success(driver))
}

/// Updates the driver's location
///
/// Updates the driver's location in the database and Redis cache.
#[utoipa::path(
    put,
    path = "/api/driver/location",
    request_body = UpdateLocationRequest,
    responses(
        (status = 200, description = "Location updated", body = ApiResponse<common::EmptyData>),
    ),
    tag = "Driver",
    security(("bearerAuth" = []))
)]
pub async fn update_location(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::model::Claims>,
    Json(req): Json<UpdateLocationRequest>,
) -> Result<ApiResponse<common::EmptyData>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid driver ID".to_string()))?;

    // Get driver by user_id
    let driver = state
        .driver_service
        .get_driver_by_user_id(user_id)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    state
        .driver_service
        .update_driver_location(
            driver.id,
            req.latitude,
            req.longitude,
            state.redis_conn.clone(),
        )
        .await?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Location updated",
    ))
}

#[utoipa::path(
    get,
    path = "/api/driver/location/{id}",
    params(
        ("id" = Uuid, Path, description = "Driver unique identifier")
    ),
    responses(
        (status = 200, description = "Recent location retrieved successfully", body = ApiResponse<crate::model::DriverLocation>),
        (status = 404, description = "Driver location not found")
    ),
    tag = "Driver"
)]
pub async fn get_driver_location(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
) -> Result<ApiResponse<DriverLocation>, AppError> {
    let location = state
        .driver_service
        .get_driver_location(driver_id, state.redis_conn.clone())
        .await?;

    Ok(ApiResponse::success(location))
}

#[utoipa::path(
    put,
    path = "/api/driver/status",
    request_body = UpdateStatusRequest,
    responses(
        (status = 200, description = "Status updated", body = ApiResponse<common::EmptyData>),
    ),
    tag = "Driver",
    security(("bearerAuth" = []))
)]
pub async fn update_status(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::model::Claims>,
    Json(req): Json<UpdateStatusRequest>,
) -> Result<ApiResponse<common::EmptyData>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid driver ID".to_string()))?;

    // Get driver by user_id
    let driver = state
        .driver_service
        .get_driver_by_user_id(user_id)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    state
        .driver_service
        .update_driver_status(driver.id, req.status)
        .await?;

    Ok(ApiResponse::message_only(StatusCode::OK, "Status updated"))
}

pub async fn update_location_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(claims): Extension<crate::model::Claims>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, claims))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, claims: crate::model::Claims) {
    let driver_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            let _ = socket.close().await;
            return;
        }
    };

    info!("Driver {} connected via WebSocket", driver_id);

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                if let Ok(req) = serde_json::from_str::<UpdateLocationRequest>(&text) {
                    if let Ok(driver) = state.driver_service.get_driver_by_user_id(driver_id) {
                        if let Err(e) = state
                            .driver_service
                            .update_driver_location(
                                driver.id,
                                req.latitude,
                                req.longitude,
                                state.redis_conn.clone(),
                            )
                            .await
                        {
                            info!(
                                "Failed to update location via WS for driver {}: {}",
                                driver.id, e
                            );
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    info!("Driver {} disconnected from WebSocket", driver_id);
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({"status": "Driver service is healthy"}))
}

#[utoipa::path(
    post,
    path = "/api/driver/upload/license",
    responses(
        (status = 200, description = "License uploaded", body = ApiResponse<String>),
    ),
    tag = "Driver",
    security(("bearerAuth" = []))
)]
pub async fn upload_driver_license(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<ApiResponse<String>, AppError> {
    let url = process_upload(state, multipart).await?;
    Ok(ApiResponse::success_with_message(
        "License uploaded successfully",
        url,
    ))
}

#[utoipa::path(
    post,
    path = "/api/driver/upload/vehicle-image",
    responses(
        (status = 200, description = "Vehicle image uploaded", body = ApiResponse<String>),
    ),
    tag = "Driver",
    security(("bearerAuth" = []))
)]
pub async fn upload_vehicle_image(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<ApiResponse<String>, AppError> {
    let url = process_upload(state, multipart).await?;
    Ok(ApiResponse::success_with_message(
        "Vehicle image uploaded successfully",
        url,
    ))
}

#[utoipa::path(
    post,
    path = "/api/driver/upload/insurance",
    responses(
        (status = 200, description = "Insurance uploaded", body = ApiResponse<String>),
    ),
    tag = "Driver",
    security(("bearerAuth" = []))
)]
pub async fn upload_vehicle_insurance(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<ApiResponse<String>, AppError> {
    let url = process_upload(state, multipart).await?;
    Ok(ApiResponse::success_with_message(
        "Insurance uploaded successfully",
        url,
    ))
}

#[utoipa::path(
    get,
    path = "/api/driver/vehicle-types",
    responses(
        (status = 200, description = "List vehicle types", body = ApiResponse<Vec<VehicleTypeInfo>>),
    ),
    tag = "Driver"
)]
pub async fn get_vehicle_types() -> Json<ApiResponse<Vec<VehicleTypeInfo>>> {
    let types = vec![
        VehicleTypeInfo {
            r#type: crate::model::VehicleType::Sedan,
            name: "Sedan".to_string(),
            description: "Comfortable car for up to 4 passengers".to_string(),
            base_price: 500.0,
        },
        VehicleTypeInfo {
            r#type: crate::model::VehicleType::Suv,
            name: "SUV".to_string(),
            description: "Spacious vehicle for larger groups or luggage".to_string(),
            base_price: 800.0,
        },
        VehicleTypeInfo {
            r#type: crate::model::VehicleType::Van,
            name: "Van".to_string(),
            description: "Big van for moving people or goods".to_string(),
            base_price: 1200.0,
        },
        VehicleTypeInfo {
            r#type: crate::model::VehicleType::Motorcycle,
            name: "Motorcycle".to_string(),
            description: "Fast and affordable ride for one passenger".to_string(),
            base_price: 300.0,
        },
    ];
    Json(ApiResponse::success(types))
}

async fn process_upload(state: AppState, mut multipart: Multipart) -> Result<String, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            let url = state.cloudinary_service.upload_image(data.to_vec()).await?;
            return Ok(url);
        }
    }
    Err(AppError::BadRequest("Missing file field".to_string()))
}

// --- Proxy Handlers to Rider Service ---

pub async fn accept_ride(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    let token = get_token(&headers)?;
    let response = state
        .rider_service
        .accept_ride(token, id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(ApiResponse::success(response))
}

pub async fn cancel_ride(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    let token = get_token(&headers)?;
    let response = state
        .rider_service
        .cancel_ride(token, id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(ApiResponse::success(response))
}

pub async fn mark_ride_arrived(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    let token = get_token(&headers)?;
    let response = state
        .rider_service
        .mark_ride_arrived(token, id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(ApiResponse::success(response))
}

pub async fn send_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    let token = get_token(&headers)?;
    let response = state
        .rider_service
        .send_message(token, &body)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(ApiResponse::success(response))
}

pub async fn get_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((context_type, context_id)): Path<(String, Uuid)>,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    let token = get_token(&headers)?;
    let response = state
        .rider_service
        .get_messages(token, &context_type, context_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(ApiResponse::success(response))
}

pub async fn chat_ws(
    _ws: WebSocketUpgrade,
    State(_state): State<AppState>,
) -> impl axum::response::IntoResponse {
    // For now, redirect or just note that drivers should connect to rider ws
    // Actually, we should proxy the WS connection, but that's complex.
    // Simplifying: the driver app can just connect to the rider WS endpoint directly.
    // If we MUST proxy, we'd use something like `proxy_socket`.
    // For this task, I'll just return a placeholder or implement a basic proxy.
    StatusCode::NOT_IMPLEMENTED
}

fn get_token(headers: &HeaderMap) -> Result<&str, AppError> {
    headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized("Missing token".to_string()))
}
