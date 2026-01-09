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
    let jwt_secret = state.driver_service.jwt_config.secret.clone();

    // Public routes (no authentication required)
    let public_routes = Router::new()
                .route("/api/driver/health", get(handlers::health_check))
        .route("/api/driver/drivers", get(handlers::list_drivers).post(handlers::create_driver))
        .route("/api/driver/drivers/{id}", get(handlers::get_driver));
       

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/api/driver/verify", post(handlers::create_driver))
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
            axum::Json(docs::DriverApiDoc::openapi())
        }))
        .with_state(state)
}




// use axum::{
//     routing::get,
//     Router,
// };
// use crate::{handlers, docs, AppState};
// use utoipa::OpenApi;

// pub fn create_routes() -> Router<AppState> {
//     Router::new()
//         .route("/api/driver/health", get(handlers::health_check))
//         .route("/api/driver/drivers", get(handlers::list_drivers).post(handlers::create_driver))
//         .route("/api/driver/drivers/{id}", get(handlers::get_driver))
//         .route("/openapi.json", get(|| async {
//             axum::Json(docs::DriverApiDoc::openapi())
//         }))
// }