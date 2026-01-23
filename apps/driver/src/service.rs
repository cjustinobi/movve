use crate::repository::DriverRepository;
use crate::model::{NewDriver, Driver};
use anyhow::Result;
use common::{
    AppError, JwtConfig,
};
// use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct DriverService {
    repo: DriverRepository,
    pub jwt_config: JwtConfig,
}

impl DriverService {
    pub fn new(repo: DriverRepository, jwt_config: JwtConfig) -> Self {
        Self { repo, jwt_config }
    }

    pub fn create_driver(&self, new_driver: NewDriver) -> Result<Driver, AppError> {
        // info!("Registering new driver: {:?}", new_driver);

        let driver = self.repo.create_driver(new_driver)
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(driver)
}


    pub fn list_drivers(&self) -> Result<Vec<Driver>> {
        self.repo.find_all().map_err(Into::into)
    }

    pub fn get_driver(&self, id: Uuid) -> Result<Driver> {
        self.repo.find_by_id(id).map_err(Into::into)
    }
}
