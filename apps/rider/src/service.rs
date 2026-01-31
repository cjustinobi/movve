use std::sync::Arc;
use uuid::Uuid;
use common::{
    AppError, JwtConfig,
};
use crate::model::{
    CreateRideRequest, DriverOption, Ride, RideEstimateRequest, RideEstimateResponse, RideResponse,
    PayRideRequest, RateDriverRequest
};
use crate::repository::RiderRepository;
use chrono::Utc;

#[derive(Clone)]
pub struct RideService {
    repository: RiderRepository,
    pub jwt_config: JwtConfig,
}

impl RideService {
    pub fn new(repository: RiderRepository, jwt_config: JwtConfig) -> Self {
        Self { repository, jwt_config }
    }

    pub async fn estimate_ride(&self, req: RideEstimateRequest) -> Result<RideEstimateResponse, AppError> {
        // Mock logic for estimation
        // In real app, we would calculate distance using a map service
        let distance = 5000.0; // 5 km
        let duration = 900.0; // 15 min
        let base_price = 50.0;
        let price = base_price + (distance / 1000.0) * 10.0; // Simple formula

        // Mock available drivers
        let drivers = vec![
            DriverOption {
                driver_id: Uuid::new_v4(),
                name: "John Doe".to_string(),
                vehicle: "Toyota Camry".to_string(),
                rating: 4.8,
                price,
                eta: 5,
            },
            DriverOption {
                driver_id: Uuid::new_v4(),
                name: "Jane Smith".to_string(),
                vehicle: "Honda Civic".to_string(),
                rating: 4.9,
                price: price * 1.1, // Premium
                eta: 3,
            },
        ];

        Ok(RideEstimateResponse {
            drivers,
            estimated_fare: price,
            distance,
            duration,
        })
    }

    pub async fn create_ride(&self, rider_id: Uuid, req: CreateRideRequest) -> Result<RideResponse, AppError> {
        let ride = Ride {
            id: Uuid::new_v4(),
            rider_id,
            driver_id: Some(req.driver_id),
            pickup: req.pickup.clone(),
            destination: req.destination.clone(),
            status: "requested".to_string(),
            fare: req.fare,
            otp: Some("1234".to_string()), // Generate random OTP in real app
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_ride = self.repository.create_ride(ride).await?;
        Ok(self.map_to_response(created_ride, distance_mock(), duration_mock()))
    }

    pub async fn get_ride(&self, ride_id: Uuid) -> Result<RideResponse, AppError> {
        let ride = self.repository.get_ride(ride_id).await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;
        
        Ok(self.map_to_response(ride, distance_mock(), duration_mock()))
    }

    pub async fn get_rider_history(&self, rider_id: Uuid) -> Result<Vec<RideResponse>, AppError> {
        let rides = self.repository.get_rides_by_rider(rider_id).await?;
        Ok(rides.into_iter().map(|r| self.map_to_response(r, distance_mock(), duration_mock())).collect())
    }

    pub async fn cancel_ride(&self, ride_id: Uuid, rider_id: Uuid) -> Result<RideResponse, AppError> {
        let ride = self.repository.get_ride(ride_id).await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;
        
        if ride.rider_id != rider_id {
            return Err(AppError::Unauthorized("Not authorized to cancel this ride".to_string()));
        }

        if ride.status != "requested" && ride.status != "accepted" {
             return Err(AppError::BadRequest("Cannot cancel ride in current status".to_string()));
        }

        let updated_ride = self.repository.update_ride_status(ride_id, "cancelled".to_string()).await?;
        Ok(self.map_to_response(updated_ride, distance_mock(), duration_mock()))
    }

    pub async fn pay_ride(&self, ride_id: Uuid, rider_id: Uuid, _req: PayRideRequest) -> Result<RideResponse, AppError> {
        let ride = self.repository.get_ride(ride_id).await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;
        
        if ride.rider_id != rider_id {
            return Err(AppError::Unauthorized("Not authorized to pay for this ride".to_string()));
        }

        let updated_ride = self.repository.update_payment_status(ride_id).await?;
        Ok(self.map_to_response(updated_ride, distance_mock(), duration_mock()))
    }

    pub async fn rate_driver(&self, ride_id: Uuid, rider_id: Uuid, req: RateDriverRequest) -> Result<(), AppError> {
         let ride = self.repository.get_ride(ride_id).await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;
        
         if ride.rider_id != rider_id {
            return Err(AppError::Unauthorized("Not authorized to rate this ride".to_string()));
        }

        // Logic to save rating would go here (e.g. RateService or update Ride table with rating)
        // For now just success
        Ok(())
    }

    fn map_to_response(&self, ride: Ride, distance: f64, duration: f64) -> RideResponse {
        RideResponse {
            id: ride.id,
            user: ride.rider_id.to_string(),
            pickup: ride.pickup,
            destination: ride.destination,
            fare: ride.fare,
            status: ride.status,
            distance,
            duration,
            otp: ride.otp,
            created_at: ride.created_at,
        }
    }
}

// Helpers
fn distance_mock() -> f64 { 5000.0 }
fn duration_mock() -> f64 { 900.0 }
