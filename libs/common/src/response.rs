use axum::{
    response::{IntoResponse, Response},
    Json,
    http::StatusCode,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub status_code: u16,
    pub message: String,
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(status_code: StatusCode, message: impl Into<String>, data: T) -> Self {
        Self {
            status_code: status_code.as_u16(),
            message: message.into(),
            data,
        }
    }

    pub fn success(data: T) -> Self {
        Self::new(StatusCode::OK, "Success", data)
    }

    pub fn success_with_message(message: impl Into<String>, data: T) -> Self {
        Self::new(StatusCode::OK, message, data)
    }

    pub fn created(message: impl Into<String>, data: T) -> Self {
        Self::new(StatusCode::CREATED, message, data)
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        (status, Json(self)).into_response()
    }
}

// Helper for responses with no data
#[derive(Serialize, ToSchema)]
pub struct EmptyData {}

impl ApiResponse<EmptyData> {
    pub fn message_only(status_code: StatusCode, message: impl Into<String>) -> Self {
        Self::new(status_code, message, EmptyData {})
    }
}