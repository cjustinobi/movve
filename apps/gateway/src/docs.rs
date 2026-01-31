use axum::{
    extract::State,
    response::Json,
};
use serde_json::{json, Value};
use utoipa::OpenApi;
use crate::AppState;

/// Minimal placeholder OpenApi for SwaggerUI initialization
/// The actual spec is loaded dynamically from /api-docs/openapi.json
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Movve API Gateway",
        version = "1.0.0",
        description = "Loading merged API documentation..."
    )
)]
pub struct GatewayApiDoc;

/// Fetch and merge OpenAPI specs from all microservices
pub async fn get_merged_openapi(
    State(state): State<AppState>,
) -> Json<Value> {
    let auth_url = format!("{}/openapi.json", state.config.services.auth_service_url);
    let driver_url = format!("{}/openapi.json", state.config.services.driver_service_url);
    let rider_url = format!("{}/openapi.json", state.config.services.rider_service_url);

    // Fetch specs from services
    let auth_spec = fetch_spec(&state.http_client, &auth_url).await;
    let driver_spec = fetch_spec(&state.http_client, &driver_url).await;
    let rider_spec = fetch_spec(&state.http_client, &rider_url).await;

    // Merge the specs
    let merged = merge_openapi_specs(auth_spec, driver_spec, rider_spec);

    Json(merged)
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

fn merge_openapi_specs(auth_spec: Option<Value>, driver_spec: Option<Value>, rider_spec: Option<Value>) -> Value {
    let mut merged = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Movve API",
            "version": "1.0.0",
            "description": "Unified API documentation for all services"
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

    // Merge rider service spec
    if let Some(spec) = rider_spec {
        merge_spec_into(&mut merged, spec, "Rider Service");
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

    tracing::info!("Merged OpenAPI spec from {}", service_name);
}