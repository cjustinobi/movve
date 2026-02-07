use crate::model::{Driver, NewDriver};
use crate::repository::DriverRepository;
use anyhow::Result;
use common::{AppError, JwtConfig};
use services::AuthServiceClient;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct DriverService {
    repo: DriverRepository,
    pub jwt_config: JwtConfig,
    auth_client: Arc<AuthServiceClient>,
}

impl DriverService {
    pub fn new(
        repo: DriverRepository,
        jwt_config: JwtConfig,
        auth_client: Arc<AuthServiceClient>,
    ) -> Self {
        Self {
            repo,
            jwt_config,
            auth_client,
        }
    }

    pub async fn create_driver(&self, new_driver: NewDriver) -> Result<Driver, AppError> {
        info!("Registering new driver: {:?}", new_driver);

        // Check if user exists in auth service
        let user_exists = self
            .auth_client
            .check_user_exists(new_driver.user_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if !user_exists {
            return Err(AppError::NotFound(format!(
                "User with ID {} not found",
                new_driver.user_id
            )));
        }

        let driver = self
            .repo
            .create_driver(new_driver)
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(driver)
    }

    pub fn list_drivers(&self) -> Result<Vec<Driver>> {
        self.repo.find_all().map_err(Into::into)
    }

    pub fn get_driver(&self, id: Uuid) -> Result<Driver> {
        self.repo.find_by_id(id).map_err(Into::into)
    }

    pub fn get_driver_by_user_id(&self, user_id: Uuid) -> Result<Driver> {
        self.repo.find_by_user_id(user_id).map_err(Into::into)
    }

    pub async fn update_driver_location(
        &self,
        driver_id: Uuid,
        latitude: f64,
        longitude: f64,
    ) -> Result<(), AppError> {
        info!(
            "Updating location for driver {}: lat={}, lon={}",
            driver_id, latitude, longitude
        );
        self.repo
            .update_location(driver_id, latitude, longitude)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }
}
