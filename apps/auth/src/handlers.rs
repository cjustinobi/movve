use axum::{extract::State, http::HeaderMap, Json};
use common::{AppError};
use crate::{
    AppState,
    model::{
        AuthResponse,
        LoginRequest, RegisterRequest, RegisterResponse,
        ForgotPasswordRequest, 
        ForgotPasswordResponse,
        Claims,
    }
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Invalid input"),
    ),
    tag = "Auth"
)]

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_service.register(req).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User logged in successfully", body = AuthResponse),
        (status = 400, description = "Invalid input"),
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_service.login(req).await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/auth/verify",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Invalid input"),
    ),
    tag = "Auth"
)]
pub async fn verify_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Claims>, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization header".to_string()))?;

    let claims = state.auth_service.verify_token(token)?;
    Ok(Json(claims))
}

#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent", body = ForgotPasswordResponse),
        (status = 404, description = "User not found"),
    ),
    tag = "Auth"
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<ForgotPasswordResponse>, AppError> {
    let token = state.auth_service.forgot_password(&req.email).await?;
    
    // TODO: In production, send email here instead of returning token
    // Example: state.email_service.send_reset_email(&req.email, &token).await?;
    
    Ok(Json(ForgotPasswordResponse {
        message: "Password reset instructions have been sent to your email".to_string(),
        token: Some(token), // Remove this in production
    }))
}