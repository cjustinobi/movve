use axum::{Json, extract::State};
use common::{ApiResponse, AppError};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{AppState, model::AuthResponse};

#[derive(Deserialize, ToSchema)]
pub struct SocialLoginPayload {
    pub provider: String,
    pub token: String,
}

/// Social Login (Google/Apple)
#[utoipa::path(
    post,
    path = "/api/auth/social-login",
    request_body = SocialLoginPayload,
    responses(
        (status = 200, description = "Logged in successfully", body = ApiResponse<AuthResponse>),
        (status = 400, description = "Invalid token or provider"),
    ),
    tag = "Auth"
)]
pub async fn social_login(
    State(state): State<AppState>,
    Json(req): Json<SocialLoginPayload>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let response = state
        .auth_service
        .social_login(&req.provider, &req.token)
        .await?;

    Ok(ApiResponse::success_with_message(
        "Logged in successfully via social provider",
        response,
    ))
}
