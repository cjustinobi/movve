use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::{AppState, docs, handlers};
use utoipa::OpenApi;

use crate::model::Claims;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::{DecodingKey, Validation, decode};

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
        .route(
            "/api/driver/drivers",
            get(handlers::list_drivers).post(handlers::create_driver),
        )
        .route("/api/driver/drivers/{id}", get(handlers::get_driver))
        .route(
            "/api/driver/vehicle-types",
            get(handlers::get_vehicle_types),
        );

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/api/driver/verify", post(handlers::create_driver))
        .route("/api/driver/location", put(handlers::update_location))
        .route("/api/driver/location/ws", get(handlers::update_location_ws))
        .route(
            "/api/driver/upload/license",
            post(handlers::upload_driver_license),
        )
        .route(
            "/api/driver/upload/vehicle-image",
            post(handlers::upload_vehicle_image),
        )
        .route(
            "/api/driver/upload/insurance",
            post(handlers::upload_vehicle_insurance),
        )
        .route("/api/driver/rides/{id}/accept", post(handlers::accept_ride))
        .route("/api/driver/rides/{id}/cancel", post(handlers::cancel_ride))
        .route("/api/driver/chat/messages", post(handlers::send_message))
        .route(
            "/api/driver/chat/conversations/{context_type}/{context_id}/messages",
            get(handlers::get_messages),
        )
        .route("/api/driver/chat/ws", get(handlers::chat_ws))
        .route_layer(middleware::from_fn(move |req, next| {
            let secret = jwt_secret.clone();
            async move { jwt_auth_middleware(secret, req, next).await }
        }));

    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // OpenAPI spec endpoint
        .route(
            "/openapi.json",
            get(|| async { axum::Json(docs::DriverApiDoc::openapi()) }),
        )
        .with_state(state)
}
