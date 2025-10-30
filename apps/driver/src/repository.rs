use diesel::prelude::*;
use diesel::result::Error;
use uuid::Uuid;

use crate::schema::drivers::dsl::*;
use crate::model::{Driver, NewDriver};
use crate::database::DbPool;

#[derive(Clone)]
pub struct DriverRepository {
    pool: DbPool,
}

impl DriverRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create_driver(&self, new_driver: NewDriver) -> Result<Driver, Error> {
        let mut conn = self.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(drivers)
            .values(&new_driver)
            .returning(Driver::as_select())
            .get_result::<Driver>(&mut conn)
    }

    pub fn find_all(&self) -> Result<Vec<Driver>, Error> {
        let mut conn = self.pool.get().expect("Failed to get DB connection");

        drivers
            .select(Driver::as_select())
            .load::<Driver>(&mut conn)
    }

    pub fn find_by_id(&self, driver_id: Uuid) -> Result<Driver, Error> {
        let mut conn = self.pool.get().expect("Failed to get DB connection");

        drivers
            .filter(id.eq(driver_id))
            .select(Driver::as_select())
            .first::<Driver>(&mut conn)
    }
}
