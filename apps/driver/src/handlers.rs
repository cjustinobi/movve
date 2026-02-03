use crate::{
    AppState,
    model::{Driver, DriverStatus, NewDriver},
};
use axum::{
    Json,
    extract::{
        Extension, Multipart, Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
};
use common::{ApiResponse, AppError};
use futures_util::SinkExt;
use serde::Deserialize;
use serde_json::json;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListDriversQuery {
    pub available: Option<bool>,
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
    let mut drivers = state
        .driver_service
        .list_drivers()
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    // Filter by available if specified
    if let Some(available) = query.available {
        drivers.retain(|d| {
            if available {
                matches!(d.status, DriverStatus::Online)
            } else {
                !matches!(d.status, DriverStatus::Online)
            }
        });
    }

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
    let driver_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid driver ID".to_string()))?;

    state
        .driver_service
        .update_driver_location(driver_id, req.latitude, req.longitude)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Location updated",
    ))
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
                    if let Err(e) = state
                        .driver_service
                        .update_driver_location(driver_id, req.latitude, req.longitude)
                        .await
                    {
                        info!(
                            "Failed to update location via WS for driver {}: {}",
                            driver_id, e
                        );
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
