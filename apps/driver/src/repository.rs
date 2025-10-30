use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::schema::drivers;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgConn = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct Driver {
    pub id: Uuid,
    pub name: String,
    pub license_number: String,
    pub phone_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = drivers)]
pub struct NewDriver {
    pub name: String,
    pub license_number: String,
    pub phone_number: Option<String>,
}

pub struct DriverRepository {
    pool: PgPool,
}

impl DriverRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<PgConn, diesel::r2d2::PoolError> {
        self.pool.get()
    }

    pub fn create_driver(&self, new_driver: NewDriver) -> Result<Driver, diesel::result::Error> {
        use crate::schema::drivers::dsl::*;
        let mut conn = self.conn().unwrap();

        diesel::insert_into(drivers)
            .values(&new_driver)
            .get_result::<Driver>(&mut conn)
    }

    pub fn find_all(&self) -> Result<Vec<Driver>, diesel::result::Error> {
        use crate::schema::drivers::dsl::*;
        let mut conn = self.conn().unwrap();
        drivers.load::<Driver>(&mut conn)
    }

    pub fn find_by_id(&self, driver_id: Uuid) -> Result<Driver, diesel::result::Error> {
        use crate::schema::drivers::dsl::*;
        let mut conn = self.conn().unwrap();
        drivers.find(driver_id).first::<Driver>(&mut conn)
    }
}
