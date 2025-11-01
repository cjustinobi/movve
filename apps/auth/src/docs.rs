use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::register,
        crate::handlers::login,
        crate::handlers::verify_token
    ),
    components(schemas(
        common::RegisterRequest,
        common::RegisterResponse,
        common::LoginRequest,
        common::AuthResponse,
        common::UserInfo,
        common::UserRole,
        common::Claims,
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