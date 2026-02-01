use std::f64::consts::PI;
use serde::{Deserialize, Serialize};
use common::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct DistanceMatrixResponse {
    pub rows: Vec<DistanceMatrixRow>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistanceMatrixRow {
    pub elements: Vec<DistanceMatrixElement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistanceMatrixElement {
    pub distance: Option<Distance>,
    pub duration: Option<Duration>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Distance {
    pub text: String,
    pub value: i64, // in meters
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Duration {
    pub text: String,
    pub value: i64, // in seconds
}

/// Service for calculating distances and routes between locations
pub struct DistanceService {
    google_api_key: Option<String>,
    http_client: reqwest::Client,
}

impl DistanceService {
    pub fn new(google_api_key: Option<String>) -> Self {
        Self {
            google_api_key,
            http_client: reqwest::Client::new(),
        }
    }

    /// Calculate distance between two points using Haversine formula (fallback)
    /// Returns distance in meters
    pub fn calculate_haversine_distance(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> f64 {
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

    /// Calculate distance and duration using Google Maps Distance Matrix API
    /// Falls back to Haversine if API is not configured or fails
    pub async fn calculate_distance_and_duration(
        &self,
        from_lat: f64,
        from_lon: f64,
        to_lat: f64,
        to_lon: f64,
    ) -> Result<(f64, f64), AppError> {
        // Try Google Maps API first if available
        if let Some(api_key) = &self.google_api_key {
            match self.google_distance_matrix(from_lat, from_lon, to_lat, to_lon, api_key).await {
                Ok((distance, duration)) => return Ok((distance, duration)),
                Err(e) => {
                    tracing::warn!("Google Maps API failed, falling back to Haversine: {}", e);
                }
            }
        }

        // Fallback to Haversine calculation
        let distance = self.calculate_haversine_distance(from_lat, from_lon, to_lat, to_lon);
        let duration = self.estimate_duration_from_distance(distance);
        Ok((distance, duration))
    }

    /// Call Google Maps Distance Matrix API
    async fn google_distance_matrix(
        &self,
        from_lat: f64,
        from_lon: f64,
        to_lat: f64,
        to_lon: f64,
        api_key: &str,
    ) -> Result<(f64, f64), AppError> {
        let url = format!(
            "https://maps.googleapis.com/maps/api/distancematrix/json?origins={},{}&destinations={},{}&key={}",
            from_lat, from_lon, to_lat, to_lon, api_key
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Google Maps API request failed: {}", e)))?;

        let data: DistanceMatrixResponse = response
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse Google Maps response: {}", e)))?;

        if data.status != "OK" {
            return Err(AppError::InternalError(format!("Google Maps API error: {}", data.status)));
        }

        let element = data.rows.first()
            .and_then(|row| row.elements.first())
            .ok_or_else(|| AppError::InternalError("No distance data returned".to_string()))?;

        if element.status != "OK" {
            return Err(AppError::InternalError(format!("Distance calculation failed: {}", element.status)));
        }

        let distance = element.distance
            .as_ref()
            .ok_or_else(|| AppError::InternalError("Distance not available".to_string()))?
            .value as f64;

        let duration = element.duration
            .as_ref()
            .ok_or_else(|| AppError::InternalError("Duration not available".to_string()))?
            .value as f64;

        Ok((distance, duration))
    }

    /// Estimate duration based on distance (fallback)
    /// Assumes average speed of 30 km/h in urban areas
    /// Returns duration in seconds
    fn estimate_duration_from_distance(&self, distance_meters: f64) -> f64 {
        let avg_speed_kmh = 30.0;
        let distance_km = distance_meters / 1000.0;
        let hours = distance_km / avg_speed_kmh;
        hours * 3600.0 // Convert to seconds
    }

    /// Calculate fare based on distance and duration
    /// Base fare + distance rate + time rate
    pub fn calculate_fare(
        &self,
        distance_meters: f64,
        duration_seconds: f64,
        vehicle_type: &str,
    ) -> f64 {
        let (base_fare, per_km_rate, per_minute_rate) = match vehicle_type {
            "economy" => (50.0, 10.0, 2.0),
            "comfort" => (75.0, 15.0, 3.0),
            "premium" => (100.0, 20.0, 5.0),
            _ => (50.0, 10.0, 2.0), // Default to economy
        };

        let distance_km = distance_meters / 1000.0;
        let duration_minutes = duration_seconds / 60.0;

        base_fare + (distance_km * per_km_rate) + (duration_minutes * per_minute_rate)
    }

    /// Calculate surge pricing multiplier based on demand
    /// In a real system, this would check current demand/supply ratio
    pub fn calculate_surge_multiplier(&self, _pickup_lat: f64, _pickup_lon: f64) -> f64 {
        // For now, return 1.0 (no surge)
        // In production, this would query a real-time demand service
        // or Redis cache with driver/rider counts per geohash
        1.0
    }

    /// Calculate ETA (Estimated Time of Arrival) in minutes
    pub fn calculate_eta(&self, distance_meters: f64) -> i32 {
        let avg_speed_kmh = 30.0;
        let distance_km = distance_meters / 1000.0;
        let hours = distance_km / avg_speed_kmh;
        let minutes = hours * 60.0;
        minutes.ceil() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        let service = DistanceService::new(None);
        
        // Distance between two points in Lagos (approximately 20-25km)
        let distance = service.calculate_haversine_distance(
            6.5244, 3.3792,  // Victoria Island
            6.4698, 3.5852,  // Lekki
        );
        
        // Should be roughly 20-25km
        assert!(distance > 20000.0 && distance < 30000.0);
    }

    #[test]
    fn test_duration_estimation() {
        let service = DistanceService::new(None);
        let distance = 5000.0; // 5km
        let duration = service.estimate_duration_from_distance(distance);
        
        // At 30km/h, 5km should take about 600 seconds (10 minutes)
        assert!(duration > 500.0 && duration < 700.0);
    }

    #[test]
    fn test_fare_calculation() {
        let service = DistanceService::new(None);
        let distance = 5000.0; // 5km
        let duration = 600.0;  // 10 minutes
        
        let fare = service.calculate_fare(distance, duration, "economy");
        
        // Base (50) + distance (5 * 10 = 50) + time (10 * 2 = 20) = 120
        assert!((fare - 120.0).abs() < 1.0);
    }

    #[test]
    fn test_eta_calculation() {
        let service = DistanceService::new(None);
        let distance = 5000.0; // 5km
        let eta = service.calculate_eta(distance);
        
        // At 30km/h, 5km should take about 10 minutes
        assert_eq!(eta, 10);
    }
}