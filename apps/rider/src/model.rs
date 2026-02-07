use chrono::{DateTime, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Jsonb;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Location with coordinates
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, AsExpression, FromSqlRow)]
#[diesel(sql_type = Jsonb)]
pub struct Location {
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
}

// Implement ToSql for Location
impl ToSql<Jsonb, Pg> for Location {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, &mut out.reborrow())
    }
}

// Implement FromSql for Location
impl FromSql<Jsonb, Pg> for Location {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
        let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
        Ok(serde_json::from_value(value)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VehicleType {
    Sedan,
    Suv,
    Van,
    Motorcycle,
}

impl std::fmt::Display for VehicleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            VehicleType::Sedan => "sedan",
            VehicleType::Suv => "suv",
            VehicleType::Van => "van",
            VehicleType::Motorcycle => "motorcycle",
        };
        write!(f, "{}", s)
    }
}

/// Request to estimate ride cost
#[derive(Debug, Deserialize, ToSchema)]
pub struct RideEstimateRequest {
    pub pickup: Location,
    pub destination: Location,
    #[serde(default = "default_vehicle_type")]
    pub vehicle_type: VehicleType,
}

fn default_vehicle_type() -> VehicleType {
    VehicleType::Sedan
}

/// Available driver option with real data
#[derive(Debug, Serialize, ToSchema)]
pub struct DriverOption {
    pub driver_id: Uuid,
    pub name: String,
    pub vehicle: String,
    pub vehicle_type: String,
    /// Vehicle type title (e.g., "Movve Go")
    pub title: String,
    /// Vehicle type tagline (e.g., "Comfortable & Reliable")
    pub tagline: String,
    /// Vehicle type description
    pub description: String,
    pub rating: f64,
    pub price: f64,
    pub eta: i32,                  // Estimated time of arrival in minutes
    pub distance_from_pickup: f64, // Distance in meters
    pub total_rides: i32,
    pub current_location: Location,
}

#[derive(
    diesel_derive_enum::DbEnum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema,
)]
#[ExistingTypePath = "crate::schema::sql_types::RideStatus"]
#[serde(rename_all = "snake_case")]
pub enum RideStatus {
    #[db_rename = "requested"]
    Requested,
    #[db_rename = "accepted"]
    Accepted,
    #[db_rename = "arrived"]
    Arrived,
    #[db_rename = "in_progress"]
    InProgress,
    #[db_rename = "stopped"]
    Stopped,
    #[db_rename = "pit_stop"]
    PitStop,
    #[db_rename = "completed"]
    Completed,
    #[db_rename = "paid"]
    Paid,
    #[db_rename = "cancelled"]
    Cancelled,
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

/// Request to create a ride
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRideRequest {
    pub driver_id: Uuid,
    pub pickup: Location,
    pub destination: Location,
    pub fare: f64,
    pub vehicle_type: VehicleType,
}

/// Request to cancel a ride
#[derive(Debug, Deserialize, ToSchema)]
pub struct CancelRideRequest {
    pub reason: String,
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

// ============================================================================
// DATABASE MODELS - SEPARATE STRUCTS FOR READ AND WRITE
// ============================================================================

/// Ride model - FOR READING FROM DATABASE (Queryable)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::rides)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Ride {
    pub id: Uuid,
    pub rider_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub pickup: Location,
    pub destination: Location,
    pub status: RideStatus,
    pub fare: f64,
    pub distance: f64,
    pub duration: f64,
    pub otp: Option<String>,
    pub cancellation_reason: Option<String>,
    pub cancelled_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// NewRide model - FOR INSERTING INTO DATABASE (Insertable)
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::rides)]
pub struct NewRide {
    pub id: Uuid,
    pub rider_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub pickup: Location,
    pub destination: Location,
    pub status: RideStatus,
    pub fare: f64,
    pub distance: f64,
    pub duration: f64,
    pub otp: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NewRide {
    /// Create a new ride request
    pub fn new(
        rider_id: Uuid,
        driver_id: Uuid,
        pickup: Location,
        destination: Location,
        fare: f64,
        distance: f64,
        duration: f64,
        otp: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            rider_id,
            driver_id: Some(driver_id),
            pickup,
            destination,
            status: RideStatus::Requested,
            fare,
            distance,
            duration,
            otp: Some(otp),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// ============================================================================
// CHAT MODELS
// ============================================================================

/// Conversation model
#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::conversations)]
pub struct Conversation {
    pub id: Uuid,
    pub context_type: String,
    pub context_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Message model
#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::messages)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub sender_role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// NewMessage for insertion
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::messages)]
pub struct NewMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub sender_role: String,
    pub content: String,
}

/// Request to send a message
#[derive(Debug, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    pub context_type: String, // "ride"
    pub context_id: Uuid,     // ride_id
    pub content: String,
}

/// Response for a message
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub sender_role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl From<Message> for MessageResponse {
    fn from(m: Message) -> Self {
        Self {
            id: m.id,
            conversation_id: m.conversation_id,
            sender_id: m.sender_id,
            sender_role: m.sender_role,
            content: m.content,
            created_at: m.created_at,
        }
    }
}

/// WebSocket Message structure
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    ChatMessage(MessageResponse),
    // Add other WS message types here
}

/// Response for ride operations
#[derive(Debug, Serialize, ToSchema)]
pub struct RideResponse {
    pub id: Uuid,
    pub user: String,
    pub pickup: Location,
    pub destination: Location,
    pub fare: f64,
    pub status: RideStatus,
    pub distance: f64,
    pub duration: f64,
    pub otp: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Ride> for RideResponse {
    fn from(ride: Ride) -> Self {
        Self {
            id: ride.id,
            user: ride.rider_id.to_string(),
            pickup: ride.pickup,
            destination: ride.destination,
            fare: ride.fare,
            status: ride.status,
            distance: ride.distance,
            duration: ride.duration,
            otp: ride.otp,
            created_at: ride.created_at,
        }
    }
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // User ID
    pub email: String,
    pub role: String,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}
