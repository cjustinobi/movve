use anyhow::Result;
use common::AppConfig;
use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone)]
pub struct DriverServiceClient {
    client: Client,
    base_url: String,
}

impl DriverServiceClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.services.driver_service_url.clone(),
        }
    }

    pub async fn get_driver_by_user_id(&self, user_id: Uuid) -> Result<Option<Value>> {
        let url = format!("{}/api/driver/drivers/by-user/{}", self.base_url, user_id);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let body: Value = resp.json().await?;
                    // The API returns ApiResponse<Driver>, so we want the 'data' field
                    Ok(body.get("data").cloned())
                } else {
                    // Driver not found or error, return None
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }
}
