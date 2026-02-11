use crate::model::{Driver, DriverLocation, DriverStatus, NewDriver};
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

    pub fn list_drivers(&self, status: Option<DriverStatus>) -> Result<Vec<Driver>> {
        self.repo.find_all(status).map_err(Into::into)
    }

    pub fn get_driver_by_user_id(&self, user_id: Uuid) -> Result<Driver> {
        self.repo.find_by_user_id(user_id).map_err(Into::into)
    }

    pub async fn update_driver_location(
        &self,
        driver_id: Uuid,
        latitude: f64,
        longitude: f64,
        redis_conn_manager: redis::aio::ConnectionManager,
    ) -> Result<(), AppError> {
        info!(
            "Updating location for driver {}: lat={}, lon={}",
            driver_id, latitude, longitude
        );
        self.repo
            .update_location(driver_id, latitude, longitude)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // Also update Redis for high-frequency access
        let mut redis_conn = redis_conn_manager.clone();
        let loc = DriverLocation {
            latitude,
            longitude,
            updated_at: chrono::Utc::now(),
        };
        let loc_json =
            serde_json::to_string(&loc).map_err(|e| AppError::InternalError(e.to_string()))?;
        let key = format!("driver:location:{}", driver_id);

        use redis::AsyncCommands;
        let _: () = redis_conn
            .set_ex(key, loc_json, 3600) // Cache for 1 hour
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(())
    }

    pub async fn update_driver_status(
        &self,
        driver_id: Uuid,
        status: DriverStatus,
    ) -> Result<(), AppError> {
        info!("Updating status for driver {}: {:?}", driver_id, status);
        self.repo
            .update_status(driver_id, status)
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    pub async fn get_driver_location(
        &self,
        driver_id: Uuid,
        mut redis_conn: redis::aio::ConnectionManager,
    ) -> Result<DriverLocation, AppError> {
        let key = format!("driver:location:{}", driver_id);
        use redis::AsyncCommands;

        // Try Redis first
        if let Ok(Some(loc_json)) = redis_conn.get::<_, Option<String>>(&key).await {
            if let Ok(loc) = serde_json::from_str::<DriverLocation>(&loc_json) {
                return Ok(loc);
            }
        }

        // Fallback to PostgreSQL
        let driver = self
            .repo
            .find_by_id(driver_id)
            .map_err(|e| AppError::NotFound(format!("Driver not found: {}", e)))?;

        let lat = driver.current_latitude.clone().unwrap_or_default();
        let lon = driver.current_longitude.clone().unwrap_or_default();

        use bigdecimal::ToPrimitive;
        Ok(DriverLocation {
            latitude: lat.to_f64().unwrap_or_default(),
            longitude: lon.to_f64().unwrap_or_default(),
            updated_at: driver.updated_at.unwrap_or_else(chrono::Utc::now),
        })
    }
}
