use axum::{extract::State, Extension, Json};
use common::{AppError, ApiResponse};
use tracing::{info, error, instrument};
use crate::{
    AppState,
    model::{
        AuthResponse, LoginRequest, RegisterRequest, RegisterResponse,
        ForgotPasswordRequest, ForgotPasswordResponse, Claims,
    }
};

/// Registers a new user
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = ApiResponse<RegisterResponse>),
        (status = 400, description = "Invalid input"),
    ),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let response = state.auth_service.register(req).await?;
    Ok(ApiResponse::success_with_message("User registered successfully", response))
}

/// Logs in an existing user
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User logged in successfully", body = ApiResponse<AuthResponse>),
        (status = 400, description = "Invalid input"),
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let response = state.auth_service.login(req).await?;
    Ok(ApiResponse::success_with_message("User logged in successfully", response))
}

/// Verifies a JWT token and returns the associated user claims
/// This endpoint is protected by JWT middleware, so if you reach here, you're authenticated
#[utoipa::path(
    get,
    path = "/api/auth/verify",
    responses(
        (status = 200, description = "User verified successfully", body = ApiResponse<Claims>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Auth",
    security(("bearerAuth" = []))
)]
pub async fn verify_token(
    Extension(claims): Extension<Claims>,
) -> Result<ApiResponse<Claims>, AppError> {
    // No need to manually verify - middleware already did it
    // Claims are injected via Extension
    Ok(ApiResponse::success_with_message("User verified successfully", claims))
}

/// Initiates the forgot password process for a user
#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent", body = ApiResponse<ForgotPasswordResponse>),
        (status = 404, description = "User not found"),
    ),
    tag = "Auth"
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<ApiResponse<ForgotPasswordResponse>, AppError> {
    let token = state.auth_service.forgot_password(&req.email).await?;
    
    // TODO: In production, send email here instead of returning token
    // Example: state.email_service.send_reset_email(&req.email, &token).await?;
    
    Ok(ApiResponse::success_with_message(
        "Password reset instructions have been sent to your email",
        ForgotPasswordResponse { token: Some(token) }
    ))
}