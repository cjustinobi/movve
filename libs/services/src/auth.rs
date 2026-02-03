use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

pub struct AuthServiceClient {
    client: Client,
    base_url: String,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    pub status_code: u16,
    pub message: String,
    pub data: T,
}

impl AuthServiceClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn check_user_exists(&self, user_id: Uuid) -> Result<bool> {
        let url = format!("{}/api/auth/users/{}", self.base_url, user_id);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(true)
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(false)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Auth service error: status={}, body={}", status, text);
            Err(anyhow!("Auth service error: {}", status))
        }
    }
}
