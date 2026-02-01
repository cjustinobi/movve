use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use uuid::Uuid;
use common::AppError;
use crate::model::Ride;
use crate::schema::rides;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct RiderRepository {
    pool: DbPool,
}

impl RiderRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create_ride(&self, ride: Ride) -> Result<Ride, AppError> {
        let mut conn = self.pool.get().map_err(|e| AppError::InternalError(e.to_string()))?;
        
        let result = diesel::insert_into(rides::table)
            .values(&ride)
            .get_result(&mut conn)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            
        Ok(result)
    }

    pub async fn get_ride(&self, ride_id: Uuid) -> Result<Option<Ride>, AppError> {
        let mut conn = self.pool.get().map_err(|e| AppError::InternalError(e.to_string()))?;
        
        let result = rides::table
            .find(ride_id)
            .first::<Ride>(&mut conn)
            .optional()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            
        Ok(result)
    }

    pub async fn get_rides_by_rider(&self, rider_id_val: Uuid) -> Result<Vec<Ride>, AppError> {
        let mut conn = self.pool.get().map_err(|e| AppError::InternalError(e.to_string()))?;
        
        // Use filter explicitly
        let result = rides::table
            .filter(rides::rider_id.eq(rider_id_val))
            .order(rides::created_at.desc())
            .load::<Ride>(&mut conn)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            
        Ok(result)
    }

    pub async fn update_ride_status(&self, ride_id: Uuid, status_val: String) -> Result<Ride, AppError> {
        let mut conn = self.pool.get().map_err(|e| AppError::InternalError(e.to_string()))?;
        
        use crate::schema::rides::dsl::*;
        use chrono::Utc;
        
        let result = diesel::update(rides.find(ride_id))
            .set((
                status.eq(status_val),
                updated_at.eq(Utc::now()),
            ))
            .get_result(&mut conn)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            
        Ok(result)
    }

    pub async fn update_payment_status(&self, ride_id: Uuid) -> Result<Ride, AppError> {
        self.update_ride_status(ride_id, "paid".to_string()).await
    }

    pub async fn update_ride_rating(&self, ride_id: Uuid, rating_value: f64, comment: Option<String>) -> Result<(), AppError> {
        let mut conn = self.pool.get().map_err(|e| AppError::InternalError(e.to_string()))?;
        
        // TODO: Add rating and comment fields to the rides table schema
        // For now, just log the rating
        tracing::info!(
            "Rating submitted for ride {}: {} stars, comment: {:?}",
            ride_id,
            rating_value,
            comment
        );
        
        Ok(())
    }
}
