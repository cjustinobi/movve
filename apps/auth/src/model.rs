use chrono::NaiveDateTime;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use serde::{Deserialize, Serialize};
use std::io::Write;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::schema::ratings;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password_hash: String,
    pub role: UserRole,
    pub avatar: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub nok_name: Option<String>,
    pub nok_phone: Option<String>,
    pub email_verified: bool,
    pub profile_completed: bool,
    #[schema(default = false)]
    pub suspended: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub profile: Option<serde_json::Value>,
}

use chrono::NaiveDate;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: NaiveDateTime,
    pub revoked: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub expires_at: NaiveDateTime,
    pub used: bool,
    pub created_at: NaiveDateTime,
}

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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: UserRole,
    pub avatar: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub nok_name: Option<String>,
    pub nok_phone: Option<String>,
    pub email_verified: bool,
    pub profile_completed: bool,
    pub profile: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ResetPasswordResponse {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdatePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct VerifyEmailRequest {
    pub code: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub nok_name: Option<String>,
    pub nok_phone: Option<String>,
    pub dob: Option<chrono::NaiveDate>,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Queryable, Selectable)]
#[diesel(table_name = ratings)]
pub struct Rating {
    pub id: Uuid,
    pub user_id: Uuid,
    pub rater_id: Uuid,
    pub rating: f64,
    pub comment: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = ratings)]
pub struct NewRating {
    pub user_id: Uuid,
    pub rater_id: Uuid,
    pub rating: f64,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateRatingRequest {
    pub rating: f64,
    pub comment: Option<String>,
}
