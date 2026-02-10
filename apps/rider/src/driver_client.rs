use crate::model::VehicleType;
use chrono::{DateTime, Utc};
use common::AppError;
use serde::{Deserialize, Deserializer, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Driver {
    pub id: Uuid,
    pub user_id: Uuid,
    pub phone: Option<String>,
    pub license_number: String,
    pub vehicle_type: VehicleType,
    pub vehicle_colour: String,
    pub vehicle_plate: String,
    pub vehicle_model: String,
    pub vehicle_year: i32,
    pub status: String,
    #[serde(deserialize_with = "deserialize_rating")]
    pub rating: Option<f64>,
    #[serde(deserialize_with = "deserialize_total_rides")]
    pub total_rides: Option<i32>,
    #[serde(deserialize_with = "deserialize_opt_f64")]
    pub current_latitude: Option<f64>,
    #[serde(deserialize_with = "deserialize_opt_f64")]
    pub current_longitude: Option<f64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverApiResponse {
    pub status_code: u16,
    pub message: String,
    pub data: Option<Driver>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriversListResponse {
    pub status_code: u16,
    pub message: String,
    pub data: Vec<Driver>,
}

/// Client for communicating with the Driver microservice
pub struct DriverClient {
    http_client: reqwest::Client,
    driver_service_url: String,
}

impl DriverClient {
    pub fn new(driver_service_url: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            driver_service_url,
        }
    }

    /// Get a specific driver by ID
    pub async fn get_driver(&self, driver_id: Uuid) -> Result<Option<Driver>, AppError> {
        let url = format!(
            "{}/api/driver/drivers/{}",
            self.driver_service_url, driver_id
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to fetch driver: {}", e)))?;

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            return Err(AppError::InternalError(format!(
                "Driver service returned error status {}: {}",
                status, body
            )));
        }

        tracing::info!("Fetched driver with status: {}", response.status());
        let response_text = response
            .text()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to read response text: {}", e)))?;

        tracing::debug!("Driver response body: {}", response_text);

        let driver_response: DriverApiResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                tracing::error!(
                    "Failed to parse driver response. Error: {}, Response: {}",
                    e,
                    response_text
                );
                AppError::InternalError(format!("Failed to parse driver response: {}", e))
            })?;

        Ok(driver_response.data)
    }

    /// Get all available drivers
    pub async fn get_available_drivers(&self) -> Result<Vec<Driver>, AppError> {
        let url = format!(
            "{}/api/driver/drivers?status=online",
            self.driver_service_url
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to fetch drivers: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            return Err(AppError::InternalError(format!(
                "Driver service returned error status {}: {}",
                status, body
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to read response text: {}", e)))?;

        tracing::debug!("Available drivers response: {}", response_text);

        let drivers_response: DriversListResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                tracing::error!(
                    "Failed to parse drivers response. Error: {}, Response: {}",
                    e,
                    response_text
                );
                AppError::InternalError(format!("Failed to parse drivers response: {}", e))
            })?;

        Ok(drivers_response.data)
    }

    /// Get drivers within a certain radius of a location
    /// Filters available drivers and calculates distance from pickup
    pub async fn get_nearby_drivers(
        &self,
        pickup_lat: f64,
        pickup_lon: f64,
        max_distance_meters: f64,
    ) -> Result<Vec<(Driver, f64)>, AppError> {
        let all_drivers = self.get_available_drivers().await?;

        let mut nearby_drivers = Vec::new();

        for driver in all_drivers {
            // Only include drivers with known locations
            if let (Some(driver_lat), Some(driver_lon)) =
                (driver.current_latitude, driver.current_longitude)
            {
                let distance = haversine_distance(pickup_lat, pickup_lon, driver_lat, driver_lon);

                if distance <= max_distance_meters {
                    nearby_drivers.push((driver, distance));
                }
            }
        }

        // Sort by distance (closest first)
        nearby_drivers.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(nearby_drivers)
    }
}

/// Calculate distance between two points using Haversine formula
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    use std::f64::consts::PI;
    let r = 6371000.0; // Earth's radius in meters

    let lat1_rad = lat1 * PI / 180.0;
    let lat2_rad = lat2 * PI / 180.0;
    let delta_lat = (lat2 - lat1) * PI / 180.0;
    let delta_lon = (lon2 - lon1) * PI / 180.0;

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}
