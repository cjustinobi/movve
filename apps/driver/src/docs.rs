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
        common::LoginResponse,
        common::VerifyResponse
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

/// Handler function to return the OpenAPI spec as JSON
/// This is called by the /openapi.json endpoint
pub fn get_openapi() -> utoipa::openapi::OpenApi {
    AuthApiDoc::openapi()
}