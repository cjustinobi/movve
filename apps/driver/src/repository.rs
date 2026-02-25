use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::prelude::*;
use diesel::result::Error;
use uuid::Uuid;

use crate::database::DbPool;
use crate::model::{Driver, DriverStatus, NewDriver, VehicleColour, VehicleTypeModel};
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
    pub vehicle_type: &'a str,
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
            vehicle_type: &new_driver.vehicle_type,
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

    pub fn find_all(
        &self,
        status_filter: Option<DriverStatus>,
        search_filter: Option<String>,
    ) -> Result<Vec<Driver>, Error> {
        let mut conn = self.pool.get().map_err(|_| Error::NotFound)?;
        let mut query = drivers.into_boxed();

        if let Some(s) = status_filter {
            query = query.filter(status.eq(s));
        }

        if let Some(s) = search_filter {
            let search_pattern = format!("%{}%", s);
            query = query.filter(
                vehicle_plate
                    .ilike(search_pattern.clone())
                    .or(vehicle_model.ilike(search_pattern.clone()))
                    .or(license_number.ilike(search_pattern)),
            );
        }

        let result = query
            .select(Driver::as_select())
            .load::<Driver>(&mut conn)?;
        Ok(result)
    }

    pub async fn get_stats(&self) -> Result<(i64, i64, i64), Error> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|_| Error::NotFound)?; // Using NotFound as generic error for pool failure
            let total = drivers.count().get_result::<i64>(&mut conn)?;
            let active = drivers
                .filter(
                    status
                        .eq(DriverStatus::Online)
                        .or(status.eq(DriverStatus::Busy)),
                ) // verified active means online/busy
                .count()
                .get_result::<i64>(&mut conn)?;
            let inactive = drivers
                .filter(status.eq(DriverStatus::Offline))
                .count()
                .get_result::<i64>(&mut conn)?;
            Ok::<_, Error>((total, active, inactive))
        })
        .await
        .map_err(|_| Error::NotFound)? // Map JoinError to Diesel Error
        // .await??
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

    pub fn get_vehicle_types(&self) -> Result<Vec<VehicleTypeModel>, Error> {
        use crate::schema::vehicle_types::dsl::*;
        let mut conn = self.pool.get().expect("Failed to get DB connection");

        vehicle_types
            .filter(is_active.eq(true))
            .select(VehicleTypeModel::as_select())
            .load::<VehicleTypeModel>(&mut conn)
    }
}
