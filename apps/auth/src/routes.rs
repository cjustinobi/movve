use axum::{
    Router, middleware, routing::{get, post}
};

use crate::{handlers, docs, AppState};
use utoipa::OpenApi;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::model::Claims;

async fn jwt_auth_middleware(
    secret: String,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add claims to request extensions
    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}

pub fn create_routes(state: AppState) -> Router {
    // Extract JWT secret from the auth service
    let jwt_secret = state.auth_service.jwt_config.secret.clone();

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/forgot-password", post(handlers::forgot_password))
        .route("/api/auth/reset-password", post(handlers::reset_password));


    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/api/auth/verify", get(handlers::verify_token))
        // Add more protected routes here as needed
        .route_layer(middleware::from_fn(move |req, next| {
            let secret = jwt_secret.clone();
            async move {
                jwt_auth_middleware(secret, req, next).await
            }
        }));

    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // OpenAPI spec endpoint
        .route("/openapi.json", get(|| async {
            axum::Json(docs::AuthApiDoc::openapi())
        }))
        .with_state(state)
}


// pub fn create_routes() -> Router<AppState> {
//     Router::new()
//         .route("/api/auth/health", get(health_check))
//         .route("/api/auth/register", post(handlers::register))
//         .route("/api/auth/login", post(handlers::login))
//         .route("/api/auth/verify", get(handlers::verify_token))
//         .route("/api/auth/forgot-password", post(handlers::forgot_password))
//         // Expose OpenAPI spec as JSON endpoint for gateway to fetch
//         .route("/openapi.json", get(|| async {
//             axum::Json(docs::AuthApiDoc::openapi())
//         }))
// }

async fn health_check() -> &'static str {
    "Auth service is healthy"
}