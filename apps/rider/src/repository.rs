use crate::model::{Conversation, Message, NewMessage, NewRide, Ride};
use crate::schema::{conversations, messages, rides};
use chrono::Utc;
use common::AppError;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use uuid::Uuid;

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
            let mut conn = pool
                .get()
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
            let mut conn = pool
                .get()
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
            let mut conn = pool
                .get()
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

    /// Get all rides for a specific driver
    pub async fn get_rides_by_driver(&self, driver_id_val: Uuid) -> Result<Vec<Ride>, AppError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|e| AppError::InternalError(format!("Failed to get connection: {}", e)))?;

            let result = rides::table
                .filter(rides::driver_id.eq(driver_id_val))
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
    pub async fn update_ride_status(
        &self,
        ride_id: Uuid,
        status_val: crate::model::RideStatus,
    ) -> Result<Ride, AppError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
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
        self.update_ride_status(ride_id, crate::model::RideStatus::Paid)
            .await
    }

    /// Cancel ride with reason
    pub async fn cancel_ride(
        &self,
        ride_id: Uuid,
        reason_val: String,
        cancelled_by_val: String,
    ) -> Result<Ride, AppError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|e| AppError::InternalError(format!("Failed to get connection: {}", e)))?;

            let result = diesel::update(rides::table.find(ride_id))
                .set((
                    rides::status.eq(crate::model::RideStatus::Cancelled),
                    rides::cancellation_reason.eq(reason_val),
                    rides::cancelled_by.eq(cancelled_by_val),
                    rides::updated_at.eq(Utc::now()),
                ))
                .get_result::<Ride>(&mut conn)
                .map_err(|e| AppError::BadRequest(format!("Failed to update ride: {}", e)))?;

            Ok(result)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    /// Update ride rating (for future implementation)
    pub async fn update_ride_rating(
        &self,
        ride_id: Uuid,
        rating_value: f64,
        comment: Option<String>,
    ) -> Result<(), AppError> {
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

    /// Get or create a conversation for a context
    pub async fn get_or_create_conversation(
        &self,
        context_type_val: String,
        context_id_val: Uuid,
    ) -> Result<Conversation, AppError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|e| AppError::InternalError(format!("Failed to get connection: {}", e)))?;

            // Try to find existing
            let existing = conversations::table
                .filter(conversations::context_type.eq(&context_type_val))
                .filter(conversations::context_id.eq(context_id_val))
                .first::<Conversation>(&mut conn)
                .optional()
                .map_err(|e| {
                    AppError::InternalError(format!("Failed to search conversation: {}", e))
                })?;

            if let Some(conv) = existing {
                return Ok(conv);
            }

            // Create new
            let new_conv = Conversation {
                id: Uuid::new_v4(),
                context_type: context_type_val,
                context_id: context_id_val,
                created_at: Utc::now(),
            };

            let result = diesel::insert_into(conversations::table)
                .values(&new_conv)
                .get_result::<Conversation>(&mut conn)
                .map_err(|e| {
                    AppError::InternalError(format!("Failed to create conversation: {}", e))
                })?;

            Ok(result)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    /// Create a new message
    pub async fn create_message(&self, new_msg: NewMessage) -> Result<Message, AppError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|e| AppError::InternalError(format!("Failed to get connection: {}", e)))?;

            let result = diesel::insert_into(messages::table)
                .values(&new_msg)
                .get_result::<Message>(&mut conn)
                .map_err(|e| AppError::InternalError(format!("Failed to save message: {}", e)))?;

            Ok(result)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    /// Get messages for a conversation
    pub async fn get_messages(&self, conversation_id_val: Uuid) -> Result<Vec<Message>, AppError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|e| AppError::InternalError(format!("Failed to get connection: {}", e)))?;

            let result = messages::table
                .filter(messages::conversation_id.eq(conversation_id_val))
                .order(messages::created_at.asc())
                .load::<Message>(&mut conn)
                .map_err(|e| AppError::InternalError(format!("Failed to fetch messages: {}", e)))?;

            Ok(result)
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }
}
