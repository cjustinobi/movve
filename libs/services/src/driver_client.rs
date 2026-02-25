use anyhow::Result;
use common::AppConfig;
use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

fn deserialize_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(f64),
    }

    let value: Option<StringOrNumber> = Option::deserialize(deserializer)?;

    match value {
        Some(StringOrNumber::String(s)) => {
            s.parse::<f64>().map(Some).map_err(serde::de::Error::custom)
        }
        Some(StringOrNumber::Number(n)) => Ok(Some(n)),
        None => Ok(None),
    }
}

// Custom deserializer for rating that handles both string and number
fn deserialize_rating<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(f64),
    }

    let value: Option<StringOrNumber> = Option::deserialize(deserializer)?;

    match value {
        Some(StringOrNumber::String(s)) => {
            s.parse::<f64>().map(Some).map_err(serde::de::Error::custom)
        }
        Some(StringOrNumber::Number(n)) => Ok(Some(n)),
        None => Ok(None),
    }
}

// Custom deserializer for total_rides that handles both string and number
fn deserialize_total_rides<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(i32),
    }

    let value: Option<StringOrNumber> = Option::deserialize(deserializer)?;

    match value {
        Some(StringOrNumber::String(s)) => {
            s.parse::<i32>().map(Some).map_err(serde::de::Error::custom)
        }
        Some(StringOrNumber::Number(n)) => Ok(Some(n)),
        None => Ok(None),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Driver {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    #[serde(default, deserialize_with = "deserialize_opt_f64")]
    pub current_latitude: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_opt_f64")]
    pub current_longitude: Option<f64>,
    pub vehicle_type: String,
    pub vehicle_make: Option<String>,
    pub vehicle_model: String,
    pub vehicle_year: i32,
    pub vehicle_colour: String,
    pub vehicle_plate: String,
    // Use custom deserializer to handle potential nulls or missing fields gracefully
    #[serde(default, deserialize_with = "deserialize_rating")]
    pub rating: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_total_rides")]
    pub total_rides: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VehicleTypeInfo {
    pub r#type: String,
    pub name: String,
    pub description: String,
    pub base_price: f64,
}

use common::response::ApiResponse;

#[derive(Clone)]
pub struct DriverServiceClient {
    client: Client,
    base_url: String,
}

impl DriverServiceClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.services.driver_service_url.clone(),
        }
    }

    pub async fn get_driver_by_user_id(&self, user_id: Uuid) -> Result<Option<Value>> {
        let url = format!("{}/api/driver/drivers/by-user/{}", self.base_url, user_id);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let body: Value = resp.json().await?;
                    // The API returns ApiResponse<Driver>, so we want the 'data' field
                    Ok(body.get("data").cloned())
                } else {
                    // Driver not found or error, return None
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Get active drivers
    pub async fn get_active_drivers(&self) -> Result<Vec<Driver>> {
        info!("Fetching active drivers");
        let url = format!("{}/api/driver/drivers?status=online", self.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch drivers: {}",
                response.status()
            ));
        }

        let api_response: ApiResponse<Vec<Driver>> = response.json().await?;
        Ok(api_response.data)
    }

    /// Get driver by ID
    pub async fn get_driver(&self, driver_id: Uuid) -> Result<Option<Driver>> {
        let url = format!("{}/api/driver/drivers/{}", self.base_url, driver_id);
        let response = self.client.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch driver: {}",
                response.status()
            ));
        }

        let api_response: ApiResponse<Driver> = response.json().await?;
        Ok(Some(api_response.data))
    }

    /// Get all vehicle types
    pub async fn get_vehicle_types(&self) -> Result<Vec<VehicleTypeInfo>> {
        let url = format!("{}/api/driver/vehicle-types", self.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch vehicle types: {}",
                response.status()
            ));
        }

        let api_response: ApiResponse<Vec<VehicleTypeInfo>> = response.json().await?;
        Ok(api_response.data)
    }
}
