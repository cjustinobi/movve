use utoipa::{OpenApi, openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme}};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::register,
        crate::handlers::login,
        crate::handlers::verify_token,
        crate::handlers::forgot_password
    ),
    modifiers(&SecurityAddon),

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

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build()
                ),
            )
        }
    }
}