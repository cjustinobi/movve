use std::sync::Arc;
use tracing::info;
use utils::generate_numeric_code;
use uuid::Uuid;
use common::{AppError, JwtConfig};
use crate::model::{
    CreateRideRequest, DriverOption, NewRide, RideEstimateRequest, RideEstimateResponse, RideResponse,
    PayRideRequest, RateDriverRequest, Location,
};
use crate::repository::RiderRepository;
use crate::driver_client::DriverClient;
use crate::distance_service::DistanceService;

#[derive(Clone)]
pub struct RiderService {
    repository: RiderRepository,
    pub jwt_config: JwtConfig,
    driver_client: Arc<DriverClient>,
    distance_service: Arc<DistanceService>,
}

impl RiderService {
    pub fn new(
        repository: RiderRepository,
        jwt_config: JwtConfig,
        driver_client: Arc<DriverClient>,
        distance_service: Arc<DistanceService>,
    ) -> Self {
        Self {
            repository,
            jwt_config,
            driver_client,
            distance_service,
        }
    }

    /// Estimate ride with real driver availability and distance calculation
    pub async fn estimate_ride(&self, req: RideEstimateRequest) -> Result<RideEstimateResponse, AppError> {
        // Calculate real distance and duration
        info!("Estimating ride from");
        let (distance, duration) = self
            .distance_service
            .calculate_distance_and_duration(
                req.pickup.latitude,
                req.pickup.longitude,
                req.destination.latitude,
                req.destination.longitude,
            )
            .await?;

        // Calculate base fare
        let base_fare = self
            .distance_service
            .calculate_fare(distance, duration, &req.vehicle_type);

        // Get surge multiplier
        let surge_multiplier = self
            .distance_service
            .calculate_surge_multiplier(req.pickup.latitude, req.pickup.longitude);

        let estimated_fare = base_fare * surge_multiplier;

        // Get nearby available drivers (within 10km radius)
        let max_distance = 10000.0; // 10km
        let nearby_drivers = self
            .driver_client
            .get_nearby_drivers(req.pickup.latitude, req.pickup.longitude, max_distance)
            .await?;

        // Filter drivers by vehicle type and convert to DriverOption
        let mut driver_options: Vec<DriverOption> = nearby_drivers
            .into_iter()
            .filter(|(driver, _)| driver.vehicle_type == req.vehicle_type)
            .map(|(driver, distance_from_pickup)| {
                // Calculate price based on driver rating (premium for higher rated drivers)
                let rating = driver.rating.as_ref().map(|r| *r).unwrap_or(0.0);
                let price_multiplier = if rating >= 4.8 {
                    1.1
                } else if rating >= 4.5 {
                    1.05
                } else {
                    1.0
                };

                let eta = self.distance_service.calculate_eta(distance_from_pickup);

                DriverOption {
                    driver_id: driver.id,
                    name: format!("Driver {}", driver.id),
                    vehicle: format!("{} {} ({})", driver.vehicle_model, driver.vehicle_year, driver.vehicle_colour),
                    vehicle_type: driver.vehicle_type.clone(),
                    rating,
                    price: estimated_fare * price_multiplier,
                    eta,
                    distance_from_pickup,
                    total_rides: driver.total_rides.unwrap_or(0),
                    current_location: Location {
                        address: "Current Location".to_string(),
                        latitude: driver.current_latitude.as_ref().map(|lat| *lat).unwrap_or(0.0),
                        longitude: driver.current_longitude.as_ref().map(|lon| *lon).unwrap_or(0.0),
                    },
                }
            })
            .collect();

        // Sort by: 1) fewer total rides (priority), 2) proximity to pickup
        driver_options.sort_by(|a, b| {
            // Primary sort: total rides (ascending - drivers with fewer rides first)
            match a.total_rides.cmp(&b.total_rides) {
                std::cmp::Ordering::Equal => {
                    // Secondary sort: distance from pickup (ascending - closer drivers first)
                    a.distance_from_pickup
                        .partial_cmp(&b.distance_from_pickup)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                other => other,
            }
        });

        Ok(RideEstimateResponse {
            drivers: driver_options,
            estimated_fare,
            distance,
            duration,
            surge_multiplier,
        })
    }

    /// Create ride with driver verification
    pub async fn create_ride(&self, rider_id: Uuid, req: CreateRideRequest) -> Result<RideResponse, AppError> {
        // Verify driver exists and is available
        let driver = self
            .driver_client
            .get_driver(req.driver_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Driver {} not found", req.driver_id)))?;

        if !driver.is_available {
            return Err(AppError::BadRequest("Driver is not available".to_string()));
        }

        // Verify driver has location
        if driver.current_latitude.is_none() || driver.current_longitude.is_none() {
            return Err(AppError::BadRequest("Driver location not available".to_string()));
        }

        // Verify vehicle type matches
        if driver.vehicle_type != req.vehicle_type {
            return Err(AppError::BadRequest(format!(
                "Driver vehicle type '{}' does not match requested type '{}'",
                driver.vehicle_type, req.vehicle_type
            )));
        }

        // Calculate actual distance and duration for this ride
        let (distance, duration) = self
            .distance_service
            .calculate_distance_and_duration(
                req.pickup.latitude,
                req.pickup.longitude,
                req.destination.latitude,
                req.destination.longitude,
            )
            .await?;

        // Generate OTP for ride verification
        let otp = generate_numeric_code(4);

        // Create NewRide for insertion
        let new_ride = NewRide::new(
            rider_id,
            req.driver_id,
            req.pickup,
            req.destination,
            req.fare,
            distance,
            duration,
            otp,
        );

        // Insert and get back the Ride
        let created_ride = self.repository.create_ride(new_ride).await?;
        
        Ok(created_ride.into())
    }

    pub async fn get_ride(&self, ride_id: Uuid) -> Result<RideResponse, AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        Ok(ride.into())
    }

    pub async fn get_rider_history(&self, rider_id: Uuid) -> Result<Vec<RideResponse>, AppError> {
        let rides = self.repository.get_rides_by_rider(rider_id).await?;
        Ok(rides.into_iter().map(|r| r.into()).collect())
    }

    pub async fn cancel_ride(&self, ride_id: Uuid, rider_id: Uuid) -> Result<RideResponse, AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        if ride.rider_id != rider_id {
            return Err(AppError::Unauthorized(
                "Not authorized to cancel this ride".to_string(),
            ));
        }

        if ride.status != "requested" && ride.status != "accepted" {
            return Err(AppError::BadRequest(
                "Cannot cancel ride in current status".to_string(),
            ));
        }

        let updated_ride = self
            .repository
            .update_ride_status(ride_id, "cancelled".to_string())
            .await?;
        
        Ok(updated_ride.into())
    }

    pub async fn pay_ride(
        &self,
        ride_id: Uuid,
        rider_id: Uuid,
        _req: PayRideRequest,
    ) -> Result<RideResponse, AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        if ride.rider_id != rider_id {
            return Err(AppError::Unauthorized(
                "Not authorized to pay for this ride".to_string(),
            ));
        }

        if ride.status != "completed" {
            return Err(AppError::BadRequest(
                "Can only pay for completed rides".to_string(),
            ));
        }

        let updated_ride = self.repository.update_payment_status(ride_id).await?;
        Ok(updated_ride.into())
    }

    pub async fn rate_driver(
        &self,
        ride_id: Uuid,
        rider_id: Uuid,
        req: RateDriverRequest,
    ) -> Result<(), AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        if ride.rider_id != rider_id {
            return Err(AppError::Unauthorized(
                "Not authorized to rate this ride".to_string(),
            ));
        }

        if ride.status != "completed" && ride.status != "paid" {
            return Err(AppError::BadRequest(
                "Can only rate completed rides".to_string(),
            ));
        }

        // Validate rating
        if req.rating < 1.0 || req.rating > 5.0 {
            return Err(AppError::BadRequest("Rating must be between 1 and 5".to_string()));
        }

        self.repository.update_ride_rating(ride_id, req.rating, req.comment).await?;

        Ok(())
    }

    /// Get current driver location for an active ride
    pub async fn get_driver_location(&self, ride_id: Uuid, rider_id: Uuid) -> Result<Location, AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        if ride.rider_id != rider_id {
            return Err(AppError::Unauthorized(
                "Not authorized to view this ride".to_string(),
            ));
        }

        let driver_id = ride
            .driver_id
            .ok_or_else(|| AppError::BadRequest("No driver assigned to this ride".to_string()))?;

        let driver = self
            .driver_client
            .get_driver(driver_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Driver not found".to_string()))?;

        Ok(Location {
            address: "Current Driver Location".to_string(),
            latitude: driver.current_latitude.unwrap_or(0.0),
            longitude: driver.current_longitude.unwrap_or(0.0),
        })
    }
}

