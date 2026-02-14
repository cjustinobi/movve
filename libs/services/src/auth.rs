use anyhow::Result;
use common::AppConfig;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserApiResponse {
    pub status_code: u16,
    pub message: String,
    pub data: User,
}

/// Client for communicating with the Auth microservice
#[derive(Clone)]
pub struct AuthServiceClient {
    http_client: reqwest::Client,
    auth_service_url: String,
}

impl AuthServiceClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            // Ensure URL doesn't end with slash for consistency
            auth_service_url: config
                .services
                .auth_service_url
                .trim_end_matches('/')
                .to_string(),
        }
    }

    /// Get a specific user by ID
    pub async fn get_user(&self, user_id: Uuid) -> Result<User> {
        let url = format!("{}/api/auth/users/{}", self.auth_service_url, user_id);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch user: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());

            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(anyhow::anyhow!("User {} not found", user_id));
            }

            return Err(anyhow::anyhow!(
                "Auth service returned error status {}: {}",
                status,
                body
            ));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response text: {}", e))?;

        let user_response: UserApiResponse = serde_json::from_str(&response_text).map_err(|e| {
            tracing::error!(
                "Failed to parse user response. Error: {}, Response: {}",
                e,
                response_text
            );
            anyhow::anyhow!("Failed to parse user response: {}", e)
        })?;

        Ok(user_response.data)
    }
    /// Check if a user exists
    pub async fn check_user_exists(&self, user_id: Uuid) -> Result<bool> {
        match self.get_user(user_id).await {
            Ok(_) => Ok(true),
            Err(e) => {
                // If error message contains "not found", return false
                if e.to_string().contains("not found") {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }
}
