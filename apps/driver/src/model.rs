use diesel::{Queryable, Selectable, Identifiable, Insertable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;
use diesel_derive_enum::DbEnum;

use crate::schema::drivers;


#[derive(DbEnum, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::DriverStatus"]
pub enum DriverStatus {
    #[db_rename = "offline"]
    Offline,
    #[db_rename = "online"]
    Online,
    #[db_rename = "busy"]
    Busy,
}

#[derive(DbEnum, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::VehicleType"]
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

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = drivers)]
pub struct Driver {
    pub id: Uuid,
    pub user_id: Uuid,
    pub phone: String,
    pub license_number: String,
    pub vehicle_type: VehicleType,
    pub vehicle_plate: String,
    pub vehicle_model: String,
    pub vehicle_year: i32,
    pub status: DriverStatus,
    pub rating: Option<BigDecimal>,
    pub total_rides: Option<i32>,
    pub current_latitude: Option<BigDecimal>,
    pub current_longitude: Option<BigDecimal>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = drivers)]
pub struct NewDriver {
    pub user_id: Uuid,
    pub phone: String,
    pub license_number: String,
    pub vehicle_type: VehicleType,
    pub vehicle_plate: String,
    pub vehicle_model: String,
    pub vehicle_year: i32,
    pub status: DriverStatus,
}