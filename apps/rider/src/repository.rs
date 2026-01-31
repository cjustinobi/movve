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
        
        let result = diesel::update(rides::table.find(ride_id))
            .set(rides::status.eq(status_val))
            .get_result(&mut conn)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            
        Ok(result)
    }

    pub async fn update_payment_status(&self, ride_id: Uuid) -> Result<Ride, AppError> {
        // Assuming we just mark it as paid or similar. For now just update status to "paid" if that's the flow, 
        // or we might need a separate field. Agent says "pay for the ride". 
        // I'll assume it changes status for now or we add a field later.
        // Let's assume paying completes the flow or sets it to 'paid'.
        self.update_ride_status(ride_id, "paid".to_string()).await
    }
}
