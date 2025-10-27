use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginRequest {
pub username: String,
pub password: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginResponse {
pub token: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Driver {
pub id: u64,
pub name: String,
}