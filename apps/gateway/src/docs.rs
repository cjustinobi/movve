use axum::{
    extract::State,
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use utoipa::OpenApi;
use crate::AppState;

/// Placeholder OpenApi struct for SwaggerUI
/// The actual merged spec is fetched dynamically
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Microservices API Gateway",
        version = "1.0.0",
        description = "Unified API documentation for all microservices. This spec is dynamically merged from all services."
    )
)]
pub struct GatewayApiDoc;

/// Fetch and merge OpenAPI specs from all microservices
pub async fn get_merged_openapi(
    State(state): State<AppState>,
) -> Result<Json<Value>, Response> {
    let auth_url = format!("{}/openapi.json", state.config.services.auth_service_url);
    let driver_url = format!("{}/openapi.json", state.config.services.driver_service_url);

    // Fetch specs from services
    let auth_spec = fetch_spec(&state.http_client, &auth_url).await;
    let driver_spec = fetch_spec(&state.http_client, &driver_url).await;

    // Merge the specs
    let merged = merge_openapi_specs(auth_spec, driver_spec);

    Ok(Json(merged))
}

async fn fetch_spec(client: &reqwest::Client, url: &str) -> Option<Value> {
    match client.get(url).send().await {
        Ok(response) => response.json::<Value>().await.ok(),
        Err(e) => {
            tracing::warn!("Failed to fetch OpenAPI spec from {}: {}", url, e);
            None
        }
    }
}

fn merge_openapi_specs(auth_spec: Option<Value>, driver_spec: Option<Value>) -> Value {
    let mut merged = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Microservices API Gateway",
            "version": "1.0.0",
            "description": "Unified API documentation for all microservices"
        },
        "servers": [
            {
                "url": "/",
                "description": "API Gateway"
            }
        ],
        "paths": {},
        "components": {
            "schemas": {},
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        },
        "tags": []
    });

    // Merge auth service spec
    if let Some(spec) = auth_spec {
        merge_spec_into(&mut merged, spec, "Auth Service");
    }

    // Merge driver service spec
    if let Some(spec) = driver_spec {
        merge_spec_into(&mut merged, spec, "Driver Service");
    }

    merged
}

fn merge_spec_into(merged: &mut Value, spec: Value, service_name: &str) {
    // Merge paths
    if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
        let merged_paths = merged["paths"].as_object_mut().unwrap();
        for (path, methods) in paths {
            merged_paths.insert(path.clone(), methods.clone());
        }
    }

    // Merge schemas
    if let Some(schemas) = spec
        .get("components")
        .and_then(|c| c.get("schemas"))
        .and_then(|s| s.as_object())
    {
        let merged_schemas = merged["components"]["schemas"].as_object_mut().unwrap();
        for (schema_name, schema) in schemas {
            merged_schemas.insert(schema_name.clone(), schema.clone());
        }
    }

    // Merge tags
    if let Some(tags) = spec.get("tags").and_then(|t| t.as_array()) {
        let merged_tags = merged["tags"].as_array_mut().unwrap();
        for tag in tags {
            merged_tags.push(tag.clone());
        }
    }

    // Add service info to description if available
    tracing::info!("Merged OpenAPI spec from {}", service_name);
}