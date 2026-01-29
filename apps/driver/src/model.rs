use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::deserialize::FromSqlRow;
use diesel::expression::AsExpression;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::schema::drivers;

#[derive(DbEnum, Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[ExistingTypePath = "crate::schema::sql_types::DriverStatus"]
#[serde(rename_all = "lowercase")]
pub enum DriverStatus {
    #[db_rename = "offline"]
    Offline,
    #[db_rename = "online"]
    Online,
    #[db_rename = "busy"]
    Busy,
}

#[derive(DbEnum, Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[ExistingTypePath = "crate::schema::sql_types::VehicleType"]
#[serde(rename_all = "lowercase")]
pub enum VehicleType {
    #[db_rename = "sedan"]
    Sedan,
    #[db_rename = "suv"]
    Suv,
    #[db_rename = "van"]
    Van,
    #[db_rename = "motorcycle"]
    Motorcycle,
}

#[derive(DbEnum, Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[ExistingTypePath = "crate::schema::sql_types::VehicleColour"]
#[serde(rename_all = "lowercase")]
pub enum VehicleColour {
    #[db_rename = "red"]
    Red,
    #[db_rename = "blue"]
    Blue,
    #[db_rename = "green"]
    Green,
    #[db_rename = "gray"]
    Gray,
    #[db_rename = "black"]
    Black,
    #[db_rename = "white"]
    White,
    #[db_rename = "silver"]
    Silver,
    #[db_rename = "yellow"]
    Yellow,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, ToSchema, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::UserRole)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Driver,
    Vendor,
    Dispatcher,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, ToSchema)]
#[diesel(table_name = drivers)]
pub struct Driver {
    pub id: Uuid,
    pub user_id: Uuid,
    pub phone: String,
    pub license_number: String,
    pub vehicle_type: VehicleType,
    pub vehicle_colour: VehicleColour,
    pub vehicle_plate: String,
    pub vehicle_model: String,
    pub vehicle_year: i32,
    pub status: DriverStatus,
    #[schema(value_type = Option<String>)]
    pub rating: Option<BigDecimal>,
    pub total_rides: Option<i32>,
    #[schema(value_type = Option<String>)]
    pub current_latitude: Option<BigDecimal>,
    #[schema(value_type = Option<String>)]
    pub current_longitude: Option<BigDecimal>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, ToSchema)]
#[diesel(table_name = drivers)]
pub struct NewDriver {
    pub user_id: Uuid,
    pub phone: String,
    pub license_number: String,
    pub vehicle_type: VehicleType,
    pub vehicle_colour: VehicleColour,
    pub vehicle_plate: String,
    pub vehicle_model: String,
    pub vehicle_year: i32,
    pub status: DriverStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
    pub iat: i64,
}
