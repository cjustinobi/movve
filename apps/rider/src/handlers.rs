use crate::{
    AppState,
    model::{
        Claims, CreateRideRequest, Location, MessageResponse, PayRideRequest, RateDriverRequest,
        RideEstimateRequest, RideEstimateResponse, RideResponse, SendMessageRequest, WsMessage,
    },
};
use axum::{
    Extension, Json,
    extract::{
        Path, State,
        ws::{Message as WsMsg, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
};
use common::{ApiResponse, AppError, EmptyData};
use futures_util::{SinkExt, StreamExt};
use tracing::info;
use uuid::Uuid;

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
    info!(
        "Estimating ride: pickup=({}, {}), destination=({}, {}), vehicle_type={}",
        req.pickup.latitude,
        req.pickup.longitude,
        req.destination.latitude,
        req.destination.longitude,
        req.vehicle_type
    );
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
    info!("Creating ride request: {:?}", req);

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    info!("User ID from token: {}", user_id);

    let response = state.rider_service.create_ride(user_id, req).await?;
    Ok(ApiResponse::success_with_message(
        "Ride requested successfully",
        response,
    ))
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
    Ok(ApiResponse::success_with_message(
        "Ride cancelled",
        response,
    ))
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
    Ok(ApiResponse::success_with_message(
        "Payment successful",
        response,
    ))
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
    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Rating submitted thank you",
    ))
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

/// Accept Ride (Driver)
#[utoipa::path(
    post,
    path = "/api/rider/rides/{id}/accept",
    responses(
        (status = 200, description = "Ride accepted", body = ApiResponse<RideResponse>),
    ),
    params(
        ("id" = Uuid, Path, description = "Ride ID")
    ),
    tag = "Driver",
    security(("bearerAuth" = []))
)]
pub async fn accept_ride(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    let driver_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid driver ID".to_string()))?;

    let response = state.rider_service.accept_ride(id, driver_id).await?;
    Ok(ApiResponse::success_with_message("Ride accepted", response))
}

/// Cancel Ride (Driver)
#[utoipa::path(
    post,
    path = "/api/rider/rides/{id}/driver-cancel",
    responses(
        (status = 200, description = "Ride cancelled by driver", body = ApiResponse<RideResponse>),
    ),
    params(
        ("id" = Uuid, Path, description = "Ride ID")
    ),
    tag = "Driver",
    security(("bearerAuth" = []))
)]
pub async fn driver_cancel_ride(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    let driver_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid driver ID".to_string()))?;

    let response = state
        .rider_service
        .driver_cancel_ride(id, driver_id)
        .await?;
    Ok(ApiResponse::success_with_message(
        "Ride cancelled by driver",
        response,
    ))
}

/// Send Message
#[utoipa::path(
    post,
    path = "/api/rider/chat/messages",
    request_body = SendMessageRequest,
    responses(
        (status = 200, description = "Message sent", body = ApiResponse<MessageResponse>),
    ),
    tag = "Chat",
    security(("bearerAuth" = []))
)]
pub async fn send_message(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SendMessageRequest>,
) -> Result<ApiResponse<MessageResponse>, AppError> {
    let sender_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let response = state
        .chat_service
        .send_message(sender_id, claims.role, req)
        .await?;
    Ok(ApiResponse::success(response))
}

/// Get Messages
#[utoipa::path(
    get,
    path = "/api/rider/chat/conversations/{context_type}/{context_id}/messages",
    responses(
        (status = 200, description = "Message history", body = ApiResponse<Vec<MessageResponse>>),
    ),
    params(
        ("context_type" = String, Path, description = "Context type (e.g., ride)"),
        ("context_id" = Uuid, Path, description = "Context ID")
    ),
    tag = "Chat",
    security(("bearerAuth" = []))
)]
pub async fn get_messages(
    State(state): State<AppState>,
    Path((context_type, context_id)): Path<(String, Uuid)>,
) -> Result<ApiResponse<Vec<MessageResponse>>, AppError> {
    let response = state
        .chat_service
        .get_messages(context_type, context_id)
        .await?;
    Ok(ApiResponse::success(response))
}

/// Chat WebSocket
pub async fn chat_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_chat_socket(socket, state))
}

async fn handle_chat_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut _receiver) = socket.split();
    let mut rx = state.chat_service.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg) {
                if sender.send(WsMsg::Text(text.into())).await.is_err() {
                    break;
                }
            }
        }
    });
}
