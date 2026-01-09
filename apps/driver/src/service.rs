use crate::repository::DriverRepository;
use crate::model::{NewDriver, Driver};
use anyhow::Result;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct DriverService {
    repo: DriverRepository,
}

impl DriverService {
    pub fn new(repo: DriverRepository) -> Self {
        Self { repo }
    }

    pub fn create_driver(&self, new_driver: NewDriver) -> Result<Driver> {
        info!("Registering new driver: {:?}", new_driver);
        let driver = self.repo.create_driver(new_driver)?;
        Ok(driver)
    }

    pub fn list_drivers(&self) -> Result<Vec<Driver>> {
        self.repo.find_all().map_err(Into::into)
    }

    pub fn get_driver(&self, id: Uuid) -> Result<Driver> {
        self.repo.find_by_id(id).map_err(Into::into)
    }
}
