use chrono::Utc;
use common::AppError;
use hex;
use reqwest;
use serde_json;
use sha1::{Digest, Sha1};

pub struct CloudinaryService {
    cloud_name: String,
    api_key: String,
    api_secret: String,
}

impl CloudinaryService {
    pub fn new(config: &common::AppConfig) -> Self {
        Self {
            cloud_name: config.cloudinary.cloudinary_cloud_name.clone(),
            api_key: config.cloudinary.cloudinary_api_key.clone(),
            api_secret: config.cloudinary.cloudinary_api_secret.clone(),
        }
    }

    pub async fn upload_file(
        &self,
        file_bytes: Vec<u8>,
        file_name: &str,
        mime_type: &str,
    ) -> Result<String, AppError> {
        let timestamp = Utc::now().timestamp();

        let params_to_sign = format!("timestamp={}{}", timestamp, self.api_secret);

        let mut hasher = Sha1::new();
        hasher.update(params_to_sign.as_bytes());
        let signature = hex::encode(hasher.finalize());

        let resource_type = if mime_type == "application/pdf" {
            "raw"
        } else {
            "image"
        };

        let url = format!(
            "https://api.cloudinary.com/v1_1/{}/{}/upload",
            self.cloud_name, resource_type
        );

        let client = reqwest::Client::new();
        let part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_name.to_string())
            .mime_str(mime_type)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let mut form = reqwest::multipart::Form::new()
            .text("api_key", self.api_key.clone())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature)
            .part("file", part);

        if resource_type == "raw" {
            form = form.text("resource_type", "raw");
        }

        let response =
            client.post(url).multipart(form).send().await.map_err(|e| {
                AppError::ExternalService(format!("Cloudinary request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalService(format!(
                "Cloudinary upload failed: {}",
                err_text
            )));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalService(format!("Failed to parse Cloudinary response: {}", e))
        })?;
        let secure_url = json["secure_url"].as_str().ok_or_else(|| {
            AppError::ExternalService("Missing secure_url in Cloudinary response".to_string())
        })?;

        Ok(secure_url.to_string())
    }

    pub async fn upload_image(&self, image_bytes: Vec<u8>) -> Result<String, AppError> {
        self.upload_file(image_bytes, "upload.jpg", "image/jpeg")
            .await
    }
}
