use axum::{
    Router, middleware,
    routing::{get, post},
    extract::Request, http::StatusCode, middleware::Next, response::Response, 
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use utoipa::OpenApi;
use crate::{AppState, docs, model::Claims, handlers};

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
    // Extract JWT secret
    // Note: In auth service config.jwt_config.secret was used.
    // Here we should probably load it from config as well.
    // But AppState needs to hold the secret or we access it via state.
    // Let's assume passed AppConfig or similar.
    // For now I'll grab it from env in main and pass it or store in state.
    // I'll add jwt_secret to AppState struct in main.rs.
    let jwt_secret = state.rider_service.jwt_config.secret.clone();

    let protected_routes = Router::new()
        .route("/api/rides/preview", post(handlers::estimate_ride))
        .route("/api/rides", post(handlers::create_ride).get(handlers::get_rides))
        .route("/api/rides/{id}", get(handlers::get_ride))
        .route("/api/rides/{id}/cancel", post(handlers::cancel_ride))
        .route("/api/rides/{id}/pay", post(handlers::pay_ride))
        .route("/api/rides/{id}/rate", post(handlers::rate_driver))
        .route("/api/rides/{id}/driver-location", get(handlers::get_driver_location))
        .route("/api/rides/{id}/status", get(handlers::get_ride_status))
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
        .with_state(state)
}
