use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    response::Response,
};
use crate::AppState;

pub async fn proxy_to_auth(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    let url = format!("{}{}", state.config.services.auth_service_url, path);

    proxy_request(state.http_client, url, method, headers, req).await
}

pub async fn proxy_to_driver(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().strip_prefix("/api/driver").unwrap_or("");
    let url = format!("{}{}", state.config.services.driver_service_url, path);

    proxy_request(state.http_client, url, method, headers, req).await
}

/// Generic proxy handler that forwards requests based on path prefix
pub async fn proxy_by_prefix(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    
    // Determine target service based on path prefix
    let (target_base_url, stripped_path) = if path.starts_with("/api/auth/") {
        (&state.config.services.auth_service_url, path)
    } else if path.starts_with("/api/driver/") {
        let stripped = path.strip_prefix("/api/driver").unwrap_or("");
        (&state.config.services.driver_service_url, stripped)
    } else {
        tracing::warn!("No matching service for path: {}", path);
        return Err(StatusCode::NOT_FOUND);
    };

    let url = format!("{}{}", target_base_url, stripped_path);
    
    proxy_request(state.http_client, url, method, headers, req).await
}

async fn proxy_request(
    client: reqwest::Client,
    url: String,
    method: Method,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, StatusCode> {
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut proxy_req = client.request(method.clone(), &url);

    // Forward relevant headers
    for (key, value) in headers.iter() {
        let key_str = key.as_str();
        if key_str != "host" && key_str != "content-length" {
            proxy_req = proxy_req.header(key, value);
        }
    }

    if !body_bytes.is_empty() {
        proxy_req = proxy_req.body(body_bytes.to_vec());
    }

    let response = proxy_req
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let response_headers = response.headers().clone();
    let response_body = response
        .bytes()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut builder = Response::builder().status(status);
    
    for (key, value) in response_headers.iter() {
        builder = builder.header(key, value);
    }

    builder
        .body(Body::from(response_body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}