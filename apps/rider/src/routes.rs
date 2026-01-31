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
    let jwt_secret = state.rider_service.jwt_config.secret.clone();

    let protected_routes = Router::new()
        .route("/api/rider/preview", post(handlers::estimate_ride))
        .route("/api/rider/rides", post(handlers::create_ride).get(handlers::get_rides))
        .route("/api/rider/{id}", get(handlers::get_ride))
        .route("/api/rider/{id}/cancel", post(handlers::cancel_ride))
        .route("/api/rider/{id}/pay", post(handlers::pay_ride))
        .route("/api/rider/{id}/rate", post(handlers::rate_driver))
        .route("/api/rider/{id}/driver-location", get(handlers::get_driver_location))
        .route("/api/rider/{id}/status", get(handlers::get_ride_status))
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
