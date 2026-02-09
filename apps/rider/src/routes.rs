use crate::{AppState, docs, handlers, model::Claims};
use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware,
    middleware::Next,
    response::Response,
    routing::{get, post},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use utoipa::OpenApi;

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
    let jwt_secret = state.rider_service.jwt_config.secret.clone();

    let protected_routes = Router::new()
        .route("/api/rider/preview", post(handlers::estimate_ride))
        .route(
            "/api/rider/rides",
            post(handlers::create_ride).get(handlers::get_rides),
        )
        .route("/api/rider/{id}", get(handlers::get_ride))
        .route("/api/rider/{id}/cancel", post(handlers::cancel_ride))
        .route("/api/rider/{id}/pay", post(handlers::pay_ride))
        .route("/api/rider/{id}/rate", post(handlers::rate_driver))
        .route(
            "/api/rider/{id}/driver-location",
            get(handlers::get_driver_location),
        )
        .route("/api/rider/{id}/status", get(handlers::get_ride_status))
        // Ride Actions (Driver)
        .route("/api/rider/rides/{id}/accept", post(handlers::accept_ride))
        .route(
            "/api/rider/rides/{id}/arrived",
            post(handlers::mark_ride_arrived),
        )
        .route("/api/rider/rides/{id}/start", post(handlers::start_ride))
        .route("/api/rider/rides/{id}/end", post(handlers::end_ride))
        .route(
            "/api/rider/rides/{id}/driver-cancel",
            post(handlers::driver_cancel_ride),
        )
        // Chat
        .route("/api/rider/chat/messages", post(handlers::send_message))
        .route(
            "/api/rider/chat/conversations/{context_type}/{context_id}/messages",
            get(handlers::get_messages),
        )
        .route("/api/rider/chat/ws", get(handlers::chat_ws))
        .route_layer(middleware::from_fn(move |req, next| {
            let secret = jwt_secret.clone();
            async move { jwt_auth_middleware(secret, req, next).await }
        }));

    Router::new()
        .merge(protected_routes)
        .route(
            "/openapi.json",
            get(|| async { axum::Json(docs::RiderApiDoc::openapi()) }),
        )
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/api/rider/docs")
                .url("/api/rider/openapi.json", docs::RiderApiDoc::openapi()),
        )
        .with_state(state)
}
