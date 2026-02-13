use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::{AppState, docs, handlers};
use utoipa::OpenApi;

use crate::model::Claims;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::{DecodingKey, Validation, decode};
use tracing::info;

async fn jwt_auth_middleware(
    secret: String,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Log presence of authorization header for debugging
    match req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
    {
        Some(hdr) => info!("Found authorization header: {}", hdr),
        None => info!("No authorization header found on request"),
    }

    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Attempt to decode and log errors for debugging
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            info!("JWT decode successful for sub: {}", token_data.claims.sub);
            // Add claims to request extensions
            req.extensions_mut().insert(token_data.claims);
            Ok(next.run(req).await)
        }
        Err(e) => {
            info!("JWT decode failed: {}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

async fn request_logger(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Log method and URI for every incoming request
    info!("Incoming request: {} {}", req.method(), req.uri());
    // Optionally log authorization header existence
    match req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
    {
        Some(hdr) => info!("Authorization header present: {}", hdr),
        None => info!("No Authorization header on incoming request"),
    }
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
            "/api/driver/drivers/by-user/{user_id}",
            get(handlers::get_driver_by_user),
        )
        .route(
            "/api/driver/vehicle-types",
            get(handlers::get_vehicle_types),
        );

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/api/driver/verify", post(handlers::create_driver))
        .route("/api/driver/location", put(handlers::update_location))
        .route(
            "/api/driver/location/{id}",
            get(handlers::get_driver_location),
        )
        .route("/api/driver/location/ws", get(handlers::update_location_ws))
        .route("/api/driver/status", put(handlers::update_status))
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
        .route("/api/driver/rides/{id}/start", post(handlers::start_ride))
        .route("/api/driver/rides/{id}/end", post(handlers::end_ride))
        .route(
            "/api/driver/rides/{id}/arrived",
            post(handlers::mark_ride_arrived),
        )
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
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/api/driver/docs")
                .url("/api/driver/openapi.json", docs::DriverApiDoc::openapi()),
        )
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit
        .layer(middleware::from_fn(request_logger))
        .with_state(state)
}
