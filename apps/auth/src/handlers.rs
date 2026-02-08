use axum::{
    Extension, Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use common::{ApiResponse, AppError, EmptyData};
use tracing::{info, info_span};
use uuid::Uuid;

use crate::{
    AppState,
    model::{
        AuthResponse, Claims, ForgotPasswordRequest, LoginRequest, LogoutRequest,
        RefreshTokenRequest, RegisterRequest, RegisterResponse, ResendVerificationRequest,
        ResetPasswordRequest, UpdatePasswordRequest, UpdateProfileRequest, User,
        VerifyEmailRequest,
    },
};

/// Gets the current user's profile
#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Current user profile", body = ApiResponse<User>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Auth",
    security(("bearerAuth" = []))
)]
pub async fn me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<ApiResponse<User>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let mut user = state.auth_service.get_user_by_id(user_id).await?;

    if matches!(user.role, crate::model::UserRole::Driver) {
        if let Ok(Some(profile)) = state.driver_service.get_driver_by_user_id(user.id).await {
            user.profile = Some(profile);
        }
    }

    Ok(ApiResponse::success(user))
}

/// Logs out a user by revoking their refresh token
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "User logged out successfully", body = ApiResponse<EmptyData>),
        (status = 400, description = "Invalid input"),
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    state.auth_service.logout(&req.refresh_token).await?;
    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Logged out successfully",
    ))
}

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
    let (response, code) = state.auth_service.register(req).await?;

    // Send welcome email asynchronously
    let state_clone = state.clone();
    let email = response.user.email.clone();
    let user_name = email.split('@').next().unwrap_or("User").to_string();
    let code_clone = code.clone();

    tokio::spawn(async move {
        if let Err(e) = state_clone
            .mail_service
            .send_welcome_email(&email, &user_name, &code_clone)
            .await
        {
            tracing::error!("Failed to send welcome email to {}: {:?}", email, e);
        }
    });

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
    let mut response = state.auth_service.login(req).await?;

    if matches!(response.user.role, crate::model::UserRole::Driver) {
        if let Ok(Some(profile)) = state
            .driver_service
            .get_driver_by_user_id(response.user.id)
            .await
        {
            response.user.profile = Some(profile);
        }
    }
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
    // send verification email code
    let code = state
        .auth_service
        .resend_verification_code(&claims.email)
        .await?;
    let user_name = claims.email.split('@').next().unwrap_or("User").to_string();
    let email = claims.email.clone();

    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = state_clone
            .mail_service
            .send_verification_code_email(&email, &user_name, &code)
            .await
        {
            tracing::error!("Failed to send verification email to {}: {:?}", email, e);
        }
    });

    Ok(ApiResponse::success_with_message(
        "Verification code has been sent to your email",
        claims,
    ))
}

/// Verifies the user's email using a 4-digit code
#[utoipa::path(
    post,
    path = "/api/auth/verify-email",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = ApiResponse<EmptyData>),
        (status = 400, description = "Invalid or expired verification code"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Auth",
)]
pub async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let user = state
        .auth_service
        .get_user_by_email(&req.email)
        .await?;

    state.auth_service.verify_email(&user.email, &req.code).await?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Email has been verified successfully.",
    ))
}

/// Resends the email verification code
#[utoipa::path(
    post,
    path = "/api/auth/resend-verification",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification code resent successfully", body = ApiResponse<EmptyData>),
        (status = 404, description = "User not found"),
    ),
    tag = "Auth"
)]
pub async fn resend_verification(
    State(state): State<AppState>,
    Json(req): Json<ResendVerificationRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    utils::validate_email(&req.email)?;
    let code = state
        .auth_service
        .resend_verification_code(&req.email)
        .await?;

    // Get user details for email
    let user = state.auth_service.get_user_by_email(&req.email).await?;
    let user_name = format!(
        "{} {}",
        user.first_name.as_deref().unwrap_or("User"),
        user.last_name.as_deref().unwrap_or("")
    );

    // Send verification email
    let state_clone = state.clone();
    let email = req.email.clone();
    let code_clone = code.clone();
    tokio::spawn(async move {
        if let Err(e) = state_clone
            .mail_service
            .send_verification_code_email(&email, &user_name, &code_clone)
            .await
        {
            tracing::error!("Failed to send verification email to {}: {:?}", email, e);
        } else {
            tracing::info!("Verification code sent to {}", email);
        }
    });

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Verification code has been sent to your email",
    ))
}

/// Initiates the forgot password process for a user
#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent", body = ApiResponse<EmptyData>),
        (status = 404, description = "User not found"),
    ),
    tag = "Auth"
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    utils::validate_email(&req.email)?;
    let token = state.auth_service.forgot_password(&req.email).await?;

    // Get user details for email
    let user = state.auth_service.get_user_by_email(&req.email).await?;
    let user_name = format!(
        "{} {}",
        user.first_name.as_deref().unwrap_or("User"),
        user.last_name.as_deref().unwrap_or("")
    );

    // Send password reset email
    let state_clone = state.clone();
    let email = req.email.clone();
    let token_clone = token.clone();
    tokio::spawn(async move {
        if let Err(e) = state_clone
            .mail_service
            .send_password_reset_email(&email, &user_name, &token_clone)
            .await
        {
            tracing::error!("Failed to send password reset email to {}: {:?}", email, e);
        } else {
            tracing::info!("Password reset email sent to {}", email);
        }
    });

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Password reset instructions have been sent to your email",
    ))
}

#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully", body = ApiResponse<EmptyData>),
        (status = 400, description = "Invalid or expired token"),
    ),
    tag = "Auth"
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    // Verify the token and get user_id
    info_span!("Reset password");
    let user_id = state
        .auth_service
        .verify_reset_token(&payload.token)
        .await?;

    // Reset the password
    state
        .auth_service
        .reset_password(&payload.token, &payload.new_password)
        .await?;

    info!("Password successfully reset for user: {}", user_id);

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Password has been reset successfully.",
    ))
}

/// Updates the password for a logged-in user
#[utoipa::path(
    post,
    path = "/api/auth/update-password",
    request_body = UpdatePasswordRequest,
    responses(
        (status = 200, description = "Password updated successfully", body = ApiResponse<EmptyData>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Auth",
    security(("bearerAuth" = []))
)]
pub async fn update_password(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdatePasswordRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    state
        .auth_service
        .update_password(user_id, &req.old_password, &req.new_password)
        .await?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Password has been updated successfully.",
    ))
}

/// Gets a user by ID (Internal use)
#[utoipa::path(
    get,
    path = "/api/auth/users/{id}",
    responses(
        (status = 200, description = "User profile", body = ApiResponse<User>),
        (status = 404, description = "User not found"),
    ),
    tag = "Auth"
)]
pub async fn get_user(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<ApiResponse<User>, AppError> {
    let user = state.auth_service.get_user_by_id(id).await?;
    Ok(ApiResponse::success(user))
}

/// Refreshes the access token using a refresh token
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = ApiResponse<AuthResponse>),
        (status = 401, description = "Invalid or expired refresh token"),
    ),
    tag = "Auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let mut response = state
        .auth_service
        .refresh_tokens(&req.refresh_token)
        .await?;

    if matches!(response.user.role, crate::model::UserRole::Driver) {
        if let Ok(Some(profile)) = state
            .driver_service
            .get_driver_by_user_id(response.user.id)
            .await
        {
            response.user.profile = Some(profile);
        }
    }
    Ok(ApiResponse::success_with_message(
        "Token refreshed successfully",
        response,
    ))
}

/// Updates the current user's profile
#[utoipa::path(
    patch,
    path = "/api/auth/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = ApiResponse<User>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Auth",
    security(("bearerAuth" = []))
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<ApiResponse<User>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let user = state.auth_service.update_profile(user_id, req).await?;
    Ok(ApiResponse::success_with_message(
        "Profile updated successfully",
        user,
    ))
}

/// Uploads an avatar image
#[utoipa::path(
    post,
    path = "/api/auth/upload/avatar",
    responses(
        (status = 200, description = "Avatar uploaded successfully", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Auth",
    security(("bearerAuth" = []))
)]
pub async fn upload_avatar(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    multipart: Multipart,
) -> Result<ApiResponse<String>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let url = process_upload(state.clone(), multipart).await?;

    // Update user's avatar in DB
    state
        .auth_service
        .update_profile(
            user_id,
            UpdateProfileRequest {
                first_name: None,
                last_name: None,
                phone: None,
                gender: None,
                nok_name: None,
                nok_phone: None,
                dob: None,
                avatar: Some(url.clone()),
            },
        )
        .await?;

    Ok(ApiResponse::success_with_message(
        "Avatar uploaded successfully",
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
