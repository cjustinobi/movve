use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::{Identifiable, Queryable, Selectable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::io::Write;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::schema::{drivers, users};

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

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, ToSchema)]
#[diesel(table_name = drivers)]
pub struct Driver {
    pub id: Uuid,
    pub user_id: Uuid,
    pub license_number: String,
    pub driver_license_image: String,
    pub insurance_number: Option<String>,
    pub insurance_image: Option<String>,
    pub vehicle_image: String,
    pub vehicle_type: String,
    pub vehicle_colour: VehicleColour,
    pub vehicle_plate: String,
    pub vehicle_model: String,
    pub vehicle_year: i32,
    pub status: DriverStatus,
    pub verified: bool,
    pub suspended: bool,
    #[schema(value_type = Option<String>)]
    pub rating: Option<BigDecimal>,
    pub total_rides: Option<i32>,
    #[schema(value_type = Option<String>)]
    pub current_latitude: Option<BigDecimal>,
    #[schema(value_type = Option<String>)]
    pub current_longitude: Option<BigDecimal>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub vehicle_verification_completed: bool,
    pub driver_license_verified: bool,
    pub insurance_verified: bool,
    pub vehicle_image_verified: bool,
    pub vehicle_capacity: i32,
}

// --- User Models (Migrated from Auth) ---

#[derive(
    Debug, Serialize, Deserialize, Copy, Clone, ToSchema, AsExpression, FromSqlRow, PartialEq,
)]
#[diesel(sql_type = crate::schema::sql_types::UserRole)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Driver,
    Vendor,
    Dispatcher,
    User,
}

impl ToSql<crate::schema::sql_types::UserRole, Pg> for UserRole {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            UserRole::Admin => out.write_all(b"admin")?,
            UserRole::Driver => out.write_all(b"driver")?,
            UserRole::Vendor => out.write_all(b"vendor")?,
            UserRole::Dispatcher => out.write_all(b"dispatcher")?,
            UserRole::User => out.write_all(b"user")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::UserRole, Pg> for UserRole {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"admin" => Ok(UserRole::Admin),
            b"driver" => Ok(UserRole::Driver),
            b"vendor" => Ok(UserRole::Vendor),
            b"dispatcher" => Ok(UserRole::Dispatcher),
            b"user" => Ok(UserRole::User),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(
    Debug, Serialize, Deserialize, Copy, Clone, ToSchema, AsExpression, FromSqlRow, PartialEq,
)]
#[diesel(sql_type = crate::schema::sql_types::Gender)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
}

impl ToSql<crate::schema::sql_types::Gender, Pg> for Gender {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Gender::Male => out.write_all(b"male")?,
            Gender::Female => out.write_all(b"female")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::Gender, Pg> for Gender {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"male" => Ok(Gender::Male),
            b"female" => Ok(Gender::Female),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable, Identifiable, ToSchema)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub avatar: Option<String>,
    pub nok_name: Option<String>,
    pub nok_phone: Option<String>,
    pub dob: Option<NaiveDate>,
    #[serde(skip)]
    pub password_hash: String,
    pub role: UserRole,
    pub email_verified: bool,
    pub profile_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub suspended: bool,
}

// --- DTOs ---

#[derive(Serialize, ToSchema)]
pub struct DriverMapLocation {
    pub id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub vehicle_type: String,
    pub vehicle_colour: VehicleColour,
    pub heading: f64,
}

#[derive(Serialize, ToSchema)]
pub struct DriverStatsResponse {
    pub total_drivers: i64,
    pub active_drivers: i64,
    pub inactive_drivers: i64,
}

#[derive(Serialize, ToSchema)]
pub struct UserStatsResponse {
    pub total_users: i64,
    pub active_users: i64,
    pub inactive_users: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub nok_name: Option<String>,
    pub nok_phone: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct SuspendUserRequest {
    pub suspended: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct ListUsersQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub role: Option<UserRole>,
    pub search: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: T,
    pub meta: serde_json::Value,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateVerificationRequest {
    pub verified: bool,
}
