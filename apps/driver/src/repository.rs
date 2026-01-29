use diesel::prelude::*;
use diesel::result::Error;
use uuid::Uuid;

use crate::database::DbPool;
use crate::model::{Driver, DriverStatus, NewDriver, VehicleColour, VehicleType};
use crate::schema::drivers::dsl::*;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::drivers)]
pub struct NewDriverDb<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub phone: &'a str,
    pub license_number: &'a str,
    pub vehicle_type: VehicleType,
    pub vehicle_colour: VehicleColour,
    pub vehicle_plate: &'a str,
    pub vehicle_model: &'a str,
    pub vehicle_year: i32,
    pub status: DriverStatus,
}

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

        let driver_db = NewDriverDb {
            id: Uuid::new_v4(),
            user_id: new_driver.user_id,
            phone: &new_driver.phone,
            license_number: &new_driver.license_number,
            vehicle_type: new_driver.vehicle_type,
            vehicle_colour: new_driver.vehicle_colour,
            vehicle_plate: &new_driver.vehicle_plate,
            vehicle_model: &new_driver.vehicle_model,
            vehicle_year: new_driver.vehicle_year,
            status: new_driver.status,
        };

        diesel::insert_into(drivers)
            .values(&driver_db)
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
