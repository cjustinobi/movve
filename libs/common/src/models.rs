use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Driver,
    Vendor,
    Dispatcher,
    User,
}

impl ToString for UserRole {
    fn to_string(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_string(),
            UserRole::Driver => "driver".to_string(),
            UserRole::Vendor => "vendor".to_string(),
            UserRole::Dispatcher => "dispatcher".to_string(),
            UserRole::User => "user".to_string(),
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