use bigdecimal::{BigDecimal, FromPrimitive};
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
    pub license_number: &'a str,
    pub driver_license_image: &'a str,
    pub vehicle_image: &'a str,
    pub insurance_image: Option<&'a str>,
    pub vehicle_type: VehicleType,
    pub vehicle_colour: VehicleColour,
    pub vehicle_plate: &'a str,
    pub vehicle_model: &'a str,
    pub vehicle_year: i32,
    pub vehicle_verification_completed: bool,
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
            license_number: &new_driver.license_number,
            driver_license_image: &new_driver.driver_license_image,
            vehicle_image: &new_driver.vehicle_image,
            insurance_image: new_driver.insurance_image.as_deref(),
            vehicle_type: new_driver.vehicle_type,
            vehicle_colour: new_driver.vehicle_colour,
            vehicle_plate: &new_driver.vehicle_plate,
            vehicle_model: &new_driver.vehicle_model,
            vehicle_year: new_driver.vehicle_year,
            vehicle_verification_completed: true,
            status: new_driver.status,
        };

        diesel::insert_into(drivers)
            .values(&driver_db)
            .returning(Driver::as_select())
            .get_result::<Driver>(&mut conn)
    }

    pub fn find_all(&self, status_filter: Option<DriverStatus>) -> Result<Vec<Driver>, Error> {
        let mut conn = self.pool.get().expect("Failed to get DB connection");

        let mut query = drivers.into_boxed();

        if let Some(s) = status_filter {
            query = query.filter(status.eq(s));
        }

        query.select(Driver::as_select()).load::<Driver>(&mut conn)
    }

    pub fn find_by_user_id(&self, uid: Uuid) -> Result<Driver, Error> {
        let mut conn = self.pool.get().expect("Failed to get DB connection");

        drivers
            .filter(user_id.eq(uid))
            .select(Driver::as_select())
            .first::<Driver>(&mut conn)
    }

    pub async fn update_location(
        &self,
        driver_id: Uuid,
        latitude: f64,
        longitude: f64,
    ) -> Result<(), Error> {
        let mut conn = self.pool.get().expect("Failed to get DB connection");

        let lat = BigDecimal::from_f64(latitude).unwrap_or_default();
        let lon = BigDecimal::from_f64(longitude).unwrap_or_default();

        diesel::update(drivers.filter(id.eq(driver_id)))
            .set((
                current_latitude.eq(Some(lat)),
                current_longitude.eq(Some(lon)),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_status(&self, driver_id: Uuid, new_status: DriverStatus) -> Result<(), Error> {
        let mut conn = self.pool.get().expect("Failed to get DB connection");

        diesel::update(drivers.filter(id.eq(driver_id)))
            .set(status.eq(new_status))
            .execute(&mut conn)?;

        Ok(())
    }
}
