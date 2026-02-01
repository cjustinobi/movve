use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use uuid::Uuid;
use common::AppError;
use crate::model::{Ride, NewRide};
use crate::schema::rides;
use chrono::Utc;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct RiderRepository {
    pool: DbPool,
}

impl RiderRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Create a new ride - uses NewRide for insert, returns Ride
    pub async fn create_ride(&self, new_ride: NewRide) -> Result<Ride, AppError> {
        let pool = self.pool.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()
                .map_err(|e| AppError::InternalError(format!("Failed to get connection: {}", e)))?;
            
            let result = diesel::insert_into(rides::table)
                .values(&new_ride)
                .get_result::<Ride>(&mut conn)
                .map_err(|e| AppError::BadRequest(format!("Failed to create ride: {}", e)))?;
                
            Ok(result)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    /// Get a single ride by ID
    pub async fn get_ride(&self, ride_id: Uuid) -> Result<Option<Ride>, AppError> {
        let pool = self.pool.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()
                .map_err(|e| AppError::InternalError(format!("Failed to get connection: {}", e)))?;
            
            let result = rides::table
                .find(ride_id)
                .select(Ride::as_select())
                .first(&mut conn)
                .optional()
                .map_err(|e| AppError::BadRequest(format!("Failed to fetch ride: {}", e)))?;
                
            Ok(result)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    /// Get all rides for a specific rider
    pub async fn get_rides_by_rider(&self, rider_id_val: Uuid) -> Result<Vec<Ride>, AppError> {
        let pool = self.pool.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()
                .map_err(|e| AppError::InternalError(format!("Failed to get connection: {}", e)))?;
            
            let result = rides::table
                .filter(rides::rider_id.eq(rider_id_val))
                .order(rides::created_at.desc())
                .select(Ride::as_select())
                .load(&mut conn)
                .map_err(|e| AppError::BadRequest(format!("Failed to fetch rides: {}", e)))?;
                
            Ok(result)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    /// Update ride status
    pub async fn update_ride_status(&self, ride_id: Uuid, status_val: String) -> Result<Ride, AppError> {
        let pool = self.pool.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()
                .map_err(|e| AppError::InternalError(format!("Failed to get connection: {}", e)))?;
            
            let result = diesel::update(rides::table.find(ride_id))
                .set((
                    rides::status.eq(status_val),
                    rides::updated_at.eq(Utc::now()),
                ))
                .get_result::<Ride>(&mut conn)
                .map_err(|e| AppError::BadRequest(format!("Failed to update ride: {}", e)))?;
                
            Ok(result)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    /// Update payment status (sets status to "paid")
    pub async fn update_payment_status(&self, ride_id: Uuid) -> Result<Ride, AppError> {
        self.update_ride_status(ride_id, "paid".to_string()).await
    }

    /// Update ride rating (for future implementation)
    pub async fn update_ride_rating(&self, ride_id: Uuid, rating_value: f64, comment: Option<String>) -> Result<(), AppError> {
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