use crate::schema::rides;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid; // Import the table module

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct RideEstimateRequest {
    pub pickup: String,
    pub destination: String,
    pub vehicle_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct RideEstimateResponse {
    pub drivers: Vec<DriverOption>,
    pub estimated_fare: f64,
    pub distance: f64,
    pub duration: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DriverOption {
    pub driver_id: Uuid,
    pub name: String,
    pub vehicle: String,
    pub rating: f32,
    pub price: f64,
    pub eta: i32, // minutes
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct CreateRideRequest {
    pub pickup: String,
    pub destination: String,
    pub driver_id: Uuid,
    pub fare: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Queryable, Selectable, Insertable)]
#[diesel(table_name = rides)]
pub struct Ride {
    pub id: Uuid,
    pub rider_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub pickup: String,
    pub destination: String,
    pub status: String, // Should be enum later
    pub fare: f64,
    pub otp: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct RideResponse {
    pub id: Uuid,
    pub user: String, // User ID
    pub pickup: String,
    pub destination: String,
    pub fare: f64,
    pub status: String,
    pub duration: f64, // seconds
    pub distance: f64, // meters
    pub otp: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct PayRideRequest {
    pub payment_method_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct RateDriverRequest {
    pub rating: i32,
    pub comment: Option<String>,
}
