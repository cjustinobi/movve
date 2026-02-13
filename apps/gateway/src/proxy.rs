use crate::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    response::Response,
};

pub async fn proxy_by_prefix(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();

    let (target_base_url, stripped_path) = if path.starts_with("/api/auth/") {
        (&state.config.services.auth_service_url, path)
    } else if path.starts_with("/api/driver/") {
        // Don't strip the prefix - send the full path
        (&state.config.services.driver_service_url, path)
    } else if path.starts_with("/api/rider/") {
        // Don't strip the prefix - send the full path
        (&state.config.services.rider_service_url, path)
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
        if key_str != "host" && key_str != "content-length" && key_str != "transfer-encoding" {
            proxy_req = proxy_req.header(key, value);
        }
    }

    if !body_bytes.is_empty() {
        proxy_req = proxy_req.body(body_bytes);
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

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};

pub async fn proxy_ws(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    req: Request,
) -> Response {
    let path = req.uri().path().to_string();

    let (target_base_url, stripped_path) = if path.starts_with("/api/auth/") {
        (&state.config.services.auth_service_url, path)
    } else if path.starts_with("/api/driver/") {
        (&state.config.services.driver_service_url, path)
    } else if path.starts_with("/api/rider/") {
        (&state.config.services.rider_service_url, path)
    } else {
        tracing::warn!("No matching service for WS path: {}", path);
        return (StatusCode::NOT_FOUND, "Service not found").into_response();
    };

    // Convert http(s) to ws(s)
    let ws_base_url = target_base_url
        .replace("https://", "wss://")
        .replace("http://", "ws://");

    let target_url = format!("{}{}", ws_base_url, stripped_path);

    tracing::info!("Proxying WebSocket to: {}", target_url);

    ws.on_upgrade(move |socket| handle_ws_socket(socket, target_url))
}

async fn handle_ws_socket(mut client_socket: WebSocket, target_url: String) {
    match tokio_tungstenite::connect_async(&target_url).await {
        Ok((mut backend_socket, _)) => {
            tracing::info!("Connected to backend WebSocket: {}", target_url);

            let (mut client_sender, mut client_receiver) = client_socket.split();
            let (mut backend_sender, mut backend_receiver) = backend_socket.split();

            let mut client_to_backend = tokio::spawn(async move {
                while let Some(Ok(msg)) = client_receiver.next().await {
                    let tungstenite_msg = match msg {
                        Message::Text(t) => {
                            tokio_tungstenite::tungstenite::Message::Text(t.to_string())
                        }
                        Message::Binary(b) => {
                            tokio_tungstenite::tungstenite::Message::Binary(b.into())
                        }
                        Message::Ping(b) => tokio_tungstenite::tungstenite::Message::Ping(b.into()),
                        Message::Pong(b) => tokio_tungstenite::tungstenite::Message::Pong(b.into()),
                        Message::Close(c) => {
                            let close_frame = c.map(|cf| tokio_tungstenite::tungstenite::protocol::CloseFrame {
                                code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::from(cf.code),
                                reason: cf.reason.to_string().into(),
                            });
                            tokio_tungstenite::tungstenite::Message::Close(close_frame)
                        }
                    };

                    if backend_sender.send(tungstenite_msg).await.is_err() {
                        break;
                    }
                }
            });

            let mut backend_to_client = tokio::spawn(async move {
                while let Some(Ok(msg)) = backend_receiver.next().await {
                    let axum_msg = match msg {
                        tokio_tungstenite::tungstenite::Message::Text(t) => Message::Text(t.into()),
                        tokio_tungstenite::tungstenite::Message::Binary(b) => {
                            Message::Binary(b.into())
                        }
                        tokio_tungstenite::tungstenite::Message::Ping(b) => Message::Ping(b.into()),
                        tokio_tungstenite::tungstenite::Message::Pong(b) => Message::Pong(b.into()),
                        tokio_tungstenite::tungstenite::Message::Close(c) => {
                            if let Some(cf) = c {
                                Message::Close(Some(axum::extract::ws::CloseFrame {
                                    code: u16::from(cf.code),
                                    reason: cf.reason.to_string().into(),
                                }))
                            } else {
                                Message::Close(None)
                            }
                        }
                        tokio_tungstenite::tungstenite::Message::Frame(_) => continue,
                    };

                    if client_sender.send(axum_msg).await.is_err() {
                        break;
                    }
                }
            });

            tokio::select! {
                _ = (&mut client_to_backend) => {},
                _ = (&mut backend_to_client) => {},
            };

            tracing::info!("WebSocket proxy finished for {}", target_url);
        }
        Err(e) => {
            tracing::error!(
                "Failed to connect to backend WebSocket {}: {}",
                target_url,
                e
            );
        }
    }
}
