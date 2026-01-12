use axum::{extract::State, Extension, Json, http::StatusCode};
use common::{ApiResponse, AppError, EmptyData};
use tracing::info;

use crate::{
    AppState,
    model::{
        AuthResponse, Claims, ForgotPasswordRequest, ForgotPasswordResponse, LoginRequest, RegisterRequest, RegisterResponse, ResetPasswordRequest
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
    // send email
    let user_name = response.user.email.split('@').next().unwrap_or("User");
    state
        .mail_service
        .send_welcome_email(&response.user.email, user_name)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(ApiResponse::success_with_message(
        "User registered successfully",
        response,
    ))
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
    Ok(ApiResponse::success_with_message(
        "User logged in successfully",
        response,
    ))
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
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<ApiResponse<Claims>, AppError> {
    // Get frontend URL from config
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let verification_link = format!("{}/verify", frontend_url);

    // send verification email
    let user_name = claims.email.split('@').next().unwrap_or("User");
    state
        .mail_service
        .send_verification_email(&claims.email, user_name, &verification_link)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(ApiResponse::success_with_message(
        "User verified successfully",
        claims,
    ))
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

    // Get user details for email
    let user = state.auth_service.get_user_by_email(&req.email).await?;
    let user_name = format!(
        "{} {}",
        user.first_name.as_deref().unwrap_or("User"),
        user.last_name.as_deref().unwrap_or("")
    );

    // Get frontend URL from config
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Send password reset email
    match state
        .mail_service
        .send_password_reset_email(&req.email, &user_name, &token, &frontend_url)
        .await
    {
        Ok(_) => tracing::info!("Password reset email sent to {}", req.email),
        Err(e) => {
            tracing::error!("Failed to send password reset email: {:?}", e);
            // TODO: queue for retry
        }
    }

    Ok(ApiResponse::success_with_message(
        "Password reset instructions have been sent to your email",
        ForgotPasswordResponse { token: Some(token) },
    ))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let mut conn = state.pool.get()?;

    // Verify the token and get user_id
    let user_id = state.auth_service.verify_reset_token(&payload.token)
        .map_err(|_| AppError::BadRequest("Invalid or expired token".to_string()))?;

    // Reset the password
    state.auth_service.reset_password(user_id, &payload.new_password)?;

    // Mark token as used
    state.auth_service.mark_token_as_used(&payload.token)?;

    info!("Password successfully reset for user: {}", user_id);

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Password has been reset successfully.",
    ))
}