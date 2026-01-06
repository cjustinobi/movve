use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: String,  // user id
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
pub struct ForgotPasswordResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>, // Only for development/testing
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