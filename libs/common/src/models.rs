use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Driver,
    Vendor,
    Dispatcher,
    User,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // user id
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
    pub iat: i64,
}