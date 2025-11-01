use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::register,
        crate::handlers::login,
        crate::handlers::verify_token,
        crate::handlers::forgot_password
    ),
    components(schemas(
        crate::model::RegisterRequest,
        crate::model::RegisterResponse,
        crate::model::LoginRequest,
        crate::model::AuthResponse,
        crate::model::UserInfo,
        crate::model::UserRole,
        crate::model::Claims,
        crate::model::ForgotPasswordRequest,
        crate::model::ResetPasswordRequest,
        crate::model::ResetPasswordResponse,
    )),
    tags(
        (name = "Auth", description = "User registration and authentication endpoints")
    ),
    info(
        title = "Auth Service API",
        version = "1.0.0",
        description = "Authentication and user management service"
    )
)]
pub struct AuthApiDoc;