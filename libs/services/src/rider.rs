use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Serialize;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct RiderServiceClient {
    client: Client,
    base_url: String,
}

impl RiderServiceClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn accept_ride(&self, token: &str, ride_id: Uuid) -> Result<serde_json::Value> {
        let url = format!("{}/api/rider/rides/{}/accept", self.base_url, ride_id);
        self.post_proxy(token, &url, &()).await
    }

    pub async fn cancel_ride(&self, token: &str, ride_id: Uuid) -> Result<serde_json::Value> {
        let url = format!(
            "{}/api/rider/rides/{}/driver-cancel",
            self.base_url, ride_id
        );
        self.post_proxy(token, &url, &()).await
    }

    pub async fn send_message(
        &self,
        token: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/rider/chat/messages", self.base_url);
        self.post_proxy(token, &url, body).await
    }

    pub async fn get_messages(
        &self,
        token: &str,
        context_type: &str,
        context_id: Uuid,
    ) -> Result<serde_json::Value> {
        let url = format!(
            "{}/api/rider/chat/conversations/{}/{}/messages",
            self.base_url, context_type, context_id
        );
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Rider service error: status={}, body={}", status, text);
            Err(anyhow!("Rider service error: {}", status))
        }
    }

    async fn post_proxy<T: Serialize>(
        &self,
        token: &str,
        url: &str,
        body: &T,
    ) -> Result<serde_json::Value> {
        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Rider service error: status={}, body={}", status, text);
            Err(anyhow!("Rider service error: {}", status))
        }
    }

    pub fn get_ws_url(&self) -> String {
        self.base_url.replace("http", "ws") + "/api/rider/chat/ws"
    }
}
