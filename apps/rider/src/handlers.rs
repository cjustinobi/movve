use crate::{
    AppState,
    model::{
        CancelRideRequest, Claims, CreateRideRequest, Location, MessageResponse, PayRideRequest,
        RateDriverRequest, RideEstimateRequest, RideEstimateResponse, RideResponse,
        SendMessageRequest,
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

/// Gets available drivers and Ride Estimate
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

/// Get Driver Ride History
#[utoipa::path(
    get,
    path = "/api/rider/driver/rides/{driver_id}",
    responses(
        (status = 200, description = "Driver ride history", body = ApiResponse<Vec<RideResponse>>),
    ),
    params(
        ("driver_id" = Uuid, Path, description = "Driver ID")
    ),
    tag = "Rider",
    security(("bearerAuth" = []))
)]
pub async fn get_driver_rides(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<RideResponse>>, AppError> {
    let response = state.rider_service.get_driver_history(driver_id).await?;
    Ok(ApiResponse::success(response))
}

/// Cancel Ride
#[utoipa::path(
    post,
    path = "/api/rider/{id}/cancel",
    request_body = CancelRideRequest,
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
    Json(req): Json<crate::model::CancelRideRequest>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let response = state
        .rider_service
        .cancel_ride(id, user_id, req.reason, "rider")
        .await?;
    Ok(ApiResponse::success_with_message(
        "Ride cancelled",
        response,
    ))
}

pub async fn start_ride(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    let driver_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid driver ID".to_string()))?;

    let response = state.rider_service.start_ride(id, driver_id).await?;
    Ok(ApiResponse::success_with_message("Ride started", response))
}

pub async fn end_ride(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    let driver_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid driver ID".to_string()))?;

    let response = state.rider_service.end_ride(id, driver_id).await?;
    Ok(ApiResponse::success_with_message(
        "Ride completed",
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

pub async fn driver_cancel_ride(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<crate::model::CancelRideRequest>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    let driver_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid driver ID".to_string()))?;

    let response = state
        .rider_service
        .driver_cancel_ride(id, driver_id, req.reason)
        .await?;
    Ok(ApiResponse::success_with_message(
        "Ride cancelled by driver",
        response,
    ))
}

pub async fn mark_ride_arrived(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<RideResponse>, AppError> {
    let driver_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid driver ID".to_string()))?;

    let (response, rider_id) = state.rider_service.mark_ride_arrived(id, driver_id).await?;

    // Send targeted notification to the rider
    state
        .notification_service
        .broadcast_ride_status_to_user(
            rider_id,
            id,
            crate::model::RideStatus::Arrived,
            "Your driver has arrived at the pickup location".to_string(),
        )
        .await;

    Ok(ApiResponse::success_with_message(
        "Marked as arrived",
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
        .notification_service
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
        .notification_service
        .get_messages(context_type, context_id)
        .await?;
    Ok(ApiResponse::success(response))
}

/// Get Valid Chat Context Types
#[utoipa::path(
    get,
    path = "/api/config/context-types",
    responses(
        (status = 200, description = "Valid context types", body = ApiResponse<Vec<String>>),
    ),
    tag = "Config"
)]
pub async fn get_chat_context_types() -> Json<ApiResponse<Vec<String>>> {
    use common::models::ChatContextType;
    let types = vec![
        ChatContextType::Ride.to_string(),
        ChatContextType::Order.to_string(),
        ChatContextType::Support.to_string(),
    ];
    Json(ApiResponse::success(types))
}

/// Chat WebSocket
pub async fn chat_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_chat_socket(socket, state, claims))
}

async fn handle_chat_socket(mut socket: WebSocket, state: AppState, claims: Claims) {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            let _ = socket.close().await;
            return;
        }
    };

    let (mut sender, mut _receiver) = socket.split();

    // Subscribe this specific user to receive notifications
    let mut rx = state.notification_service.subscribe_user(user_id).await;

    info!("User {} connected to WebSocket", user_id);

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg) {
                if sender.send(WsMsg::Text(text.into())).await.is_err() {
                    break;
                }
            }
        }
        info!("User {} disconnected from WebSocket", user_id);
    });

    // Cleanup when user disconnects
    state.notification_service.unsubscribe_user(user_id).await;
}
