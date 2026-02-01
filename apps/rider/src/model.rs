use chrono::{DateTime, Utc};
use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid; // Import the table module

/// Location with coordinates
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub struct Location {
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
}

// Diesel JSON serialization
impl<DB> diesel::serialize::ToSql<diesel::sql_types::Jsonb, DB> for Location
where
    DB: diesel::backend::Backend,
    serde_json::Value: diesel::serialize::ToSql<diesel::sql_types::Jsonb, DB>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        let value = serde_json::to_value(self)?;
        <serde_json::Value as diesel::serialize::ToSql<diesel::sql_types::Jsonb, DB>>::to_sql(&value, &mut out.reborrow())
    }
}

impl<DB> diesel::deserialize::FromSql<diesel::sql_types::Jsonb, DB> for Location
where
    DB: diesel::backend::Backend,
    serde_json::Value: diesel::deserialize::FromSql<diesel::sql_types::Jsonb, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let value = serde_json::Value::from_sql(bytes)?;
        Ok(serde_json::from_value(value)?)
    }
}

fn default_vehicle_type() -> String {
    "sedan".to_string()
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // User ID
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RideEstimateRequest {
    pub pickup: Location,
    pub destination: Location,
    #[serde(default = "default_vehicle_type")]
    pub vehicle_type: String,
}

/// Response for ride estimation
#[derive(Debug, Serialize, ToSchema)]
pub struct RideEstimateResponse {
    pub drivers: Vec<DriverOption>,
    pub estimated_fare: f64,
    pub distance: f64, // in meters
    pub duration: f64, // in seconds
    pub surge_multiplier: f64,
}


/// Available driver option with real data
#[derive(Debug, Serialize, ToSchema)]
pub struct DriverOption {
    pub driver_id: Uuid,
    pub name: String,
    pub vehicle: String,
    pub vehicle_type: String,
    pub rating: f64,
    pub price: f64,
    pub eta: i32, // Estimated time of arrival in minutes
    pub distance_from_pickup: f64, // Distance in meters
    pub total_rides: i32,
    pub current_location: Location,
}

/// Request to create a ride
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRideRequest {
    pub driver_id: Uuid,
    pub pickup: Location,
    pub destination: Location,
    pub fare: f64,
    pub vehicle_type: String,
}

/// Ride database model
#[derive(Debug, Clone, Queryable, Insertable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::rides)]
pub struct Ride {
    pub id: Uuid,
    pub rider_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub pickup: Location,
    pub destination: Location,
    pub status: String,
    pub fare: f64,
    pub distance: f64,    // Store actual distance
    pub duration: f64,    // Store actual duration
    pub otp: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for ride operations
#[derive(Debug, Serialize, ToSchema)]
pub struct RideResponse {
    pub id: Uuid,
    pub user: String,
    pub pickup: Location,
    pub destination: Location,
    pub fare: f64,
    pub status: String,
    pub distance: f64,
    pub duration: f64,
    pub otp: Option<String>,
    pub created_at: DateTime<Utc>,
}


/// Request to pay for a ride
#[derive(Debug, Deserialize, ToSchema)]
pub struct PayRideRequest {
    pub payment_method: String, // "card", "cash", "wallet"
    pub amount: f64,
}

/// Request to rate a driver
#[derive(Debug, Deserialize, ToSchema)]
pub struct RateDriverRequest {
    pub rating: f64, // 1-5
    pub comment: Option<String>,
}