use crate::distance_service::DistanceService;
use crate::model::{
    CreateRideRequest, DriverOption, Location, NewRide, PayRideRequest, RateDriverRequest,
    RideDriver, RideEstimateRequest, RideEstimateResponse, RideResponse,
};
use crate::repository::RiderRepository;
use common::{AppError, JwtConfig};
use services::auth::AuthServiceClient;
use services::driver_client::DriverServiceClient;
use std::sync::Arc;

use utils::generate_numeric_code;
use uuid::Uuid;

#[derive(Clone)]
pub struct RiderService {
    repository: RiderRepository,
    pub jwt_config: JwtConfig,
    driver_client: Arc<DriverServiceClient>,
    auth_client: Arc<AuthServiceClient>,
    distance_service: Arc<DistanceService>,
}

impl RiderService {
    pub fn new(
        repository: RiderRepository,
        jwt_config: JwtConfig,
        driver_client: Arc<DriverServiceClient>,
        auth_client: Arc<AuthServiceClient>,
        distance_service: Arc<DistanceService>,
    ) -> Self {
        Self {
            repository,
            jwt_config,
            driver_client,
            auth_client,
            distance_service,
        }
    }

    /// Estimate ride with real driver availability and distance calculation
    pub async fn estimate_ride(
        &self,
        req: RideEstimateRequest,
    ) -> Result<RideEstimateResponse, AppError> {
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

        // Fetch active drivers
        let drivers = self
            .driver_client
            .get_active_drivers()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        // Filter drivers by vehicle type and convert to DriverOption
        let mut driver_options = Vec::new();

        for driver in drivers {
            // Match vehicle type
            let type_match = match (&driver.vehicle_type, &req.vehicle_type) {
                (services::driver_client::VehicleType::Sedan, crate::model::VehicleType::Sedan) => {
                    true
                }
                (services::driver_client::VehicleType::Suv, crate::model::VehicleType::Suv) => true,
                (services::driver_client::VehicleType::Van, crate::model::VehicleType::Van) => true,
                (
                    services::driver_client::VehicleType::Motorcycle,
                    crate::model::VehicleType::Motorcycle,
                ) => true,
                _ => false,
            };

            if !type_match {
                continue;
            }

            // Calculate distance
            let (lat, lon) = match (driver.current_latitude, driver.current_longitude) {
                (Some(lat), Some(lon)) => (lat, lon),
                _ => continue,
            };

            let distance_from_pickup = self.distance_service.calculate_haversine_distance(
                req.pickup.latitude,
                req.pickup.longitude,
                lat,
                lon,
            );

            if distance_from_pickup > max_distance {
                continue;
            }

            // Fetch user info from Auth service
            // We ignore errors here and just log them, maybe fallback to default values
            // or skip the driver if critical info is missing?
            // For now, let's try to fetch and if it fails, we just don't populate the extra fields
            let user_info = match self.auth_client.get_user(driver.user_id).await {
                Ok(user) => Some(user),
                Err(e) => {
                    tracing::error!(
                        "Failed to fetch user info for driver {}: {}",
                        driver.user_id,
                        e
                    );
                    None
                }
            };

            let rating = driver.rating.as_ref().map(|r| *r).unwrap_or(0.0);
            let price_multiplier = if rating >= 4.8 {
                1.1
            } else if rating >= 4.5 {
                1.05
            } else {
                1.0
            };

            let eta = self.distance_service.calculate_eta(distance_from_pickup);

            let (title, tagline, description) = match driver.vehicle_type {
                services::driver_client::VehicleType::Sedan => (
                    "Movve Go",
                    "Comfortable & Reliable",
                    "Affordable, everyday rides for up to 4 people",
                ),
                services::driver_client::VehicleType::Suv => (
                    "Movve XL",
                    "Spacious & Premium",
                    "Spacious vehicles with more legroom or luggage space",
                ),
                services::driver_client::VehicleType::Van => (
                    "Movve Van",
                    "Extra Space for Everyone",
                    "Large vehicles for groups of up to 6 people or extra luggage",
                ),
                services::driver_client::VehicleType::Motorcycle => (
                    "Movve Moto",
                    "Fast & Affordable",
                    "Fast and nimble rides for solo travelers",
                ),
            };

            let (first_name, last_name, avatar) = if let Some(user) = user_info {
                (user.first_name, user.last_name, user.avatar)
            } else {
                (None, None, None)
            };

            driver_options.push(DriverOption {
                driver_id: driver.user_id,
                name: format!("Driver {}", driver.user_id), // Fallback or keep as is?
                first_name,
                last_name,
                avatar,
                vehicle: format!(
                    "{} {} ({})",
                    driver.vehicle_model, driver.vehicle_year, driver.vehicle_colour
                ),
                vehicle_type: driver.vehicle_type.to_string(),
                title: title.to_string(),
                tagline: tagline.to_string(),
                description: description.to_string(),
                rating,
                price: estimated_fare * price_multiplier,
                eta,
                distance_from_pickup,
                total_rides: driver.total_rides.unwrap_or(0),
                current_location: Location {
                    address: "Current Location".to_string(),
                    latitude: lat,
                    longitude: lon,
                },
            });
        }

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
    pub async fn create_ride(
        &self,
        rider_id: Uuid,
        req: CreateRideRequest,
    ) -> Result<RideResponse, AppError> {
        // Verify driver exists and is available
        // Fetch driver details if provided
        let driver = self
            .driver_client
            .get_driver(req.driver_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Driver {} not found", req.driver_id)))?;

        // Verify driver has location
        if driver.current_latitude.is_none() || driver.current_longitude.is_none() {
            return Err(AppError::BadRequest(
                "Driver location not available".to_string(),
            ));
        }

        // Verify vehicle type matches
        let type_match = match (&driver.vehicle_type, &req.vehicle_type) {
            (services::driver_client::VehicleType::Sedan, crate::model::VehicleType::Sedan) => true,
            (services::driver_client::VehicleType::Suv, crate::model::VehicleType::Suv) => true,
            (services::driver_client::VehicleType::Van, crate::model::VehicleType::Van) => true,
            (
                services::driver_client::VehicleType::Motorcycle,
                crate::model::VehicleType::Motorcycle,
            ) => true,
            _ => false,
        };

        if !type_match {
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

        self.enrich_ride_response(created_ride).await
    }

    pub async fn get_ride(&self, ride_id: Uuid) -> Result<RideResponse, AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        self.enrich_ride_response(ride).await
    }

    pub async fn get_rider_history(&self, rider_id: Uuid) -> Result<Vec<RideResponse>, AppError> {
        let rides = self.repository.get_rides_by_rider(rider_id).await?;
        let mut responses = Vec::new();
        for ride in rides {
            responses.push(self.enrich_ride_response(ride).await?);
        }
        Ok(responses)
    }

    pub async fn get_driver_history(&self, driver_id: Uuid) -> Result<Vec<RideResponse>, AppError> {
        let rides = self.repository.get_rides_by_driver(driver_id).await?;
        let mut responses = Vec::new();
        for ride in rides {
            responses.push(self.enrich_ride_response(ride).await?);
        }
        Ok(responses)
    }

    pub async fn cancel_ride(
        &self,
        ride_id: Uuid,
        user_id: Uuid,
        reason: String,
        cancelled_by_role: &str,
    ) -> Result<RideResponse, AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        // If rider, verify ownership
        if cancelled_by_role == "rider" && ride.rider_id != user_id {
            return Err(AppError::Unauthorized(
                "Not authorized to cancel this ride".to_string(),
            ));
        }

        // If driver, verify assignment
        if cancelled_by_role == "driver" && ride.driver_id != Some(user_id) {
            return Err(AppError::Unauthorized(
                "Not authorized to cancel this ride".to_string(),
            ));
        }

        use crate::model::RideStatus;

        match ride.status {
            RideStatus::Requested | RideStatus::Accepted | RideStatus::Arrived => {}
            _ => {
                return Err(AppError::BadRequest(
                    "Cannot cancel ride in current status".to_string(),
                ));
            }
        }

        let updated_ride = self
            .repository
            .cancel_ride(ride_id, reason, cancelled_by_role.to_string())
            .await?;

        self.enrich_ride_response(updated_ride).await
    }

    async fn enrich_ride_response(
        &self,
        ride: crate::model::Ride,
    ) -> Result<RideResponse, AppError> {
        let mut response: RideResponse = ride.clone().into();

        if let Some(driver_id) = ride.driver_id {
            // Fetch driver details
            if let Ok(Some(driver)) = self.driver_client.get_driver(driver_id).await {
                // Fetch user details for driver
                let user = self.auth_client.get_user(driver_id).await.ok();

                let (name, avatar, phone) = if let Some(u) = user {
                    (
                        format!(
                            "{} {}",
                            u.first_name.unwrap_or_default(),
                            u.last_name.unwrap_or_default()
                        )
                        .trim()
                        .to_string(),
                        u.avatar,
                        u.phone,
                    )
                } else {
                    (format!("Driver {}", driver_id), None, None)
                };

                // If name is empty (e.g. user had no first/last name), fallback
                let name = if name.is_empty() {
                    format!("Driver {}", driver_id)
                } else {
                    name
                };

                let rating = driver.rating.as_ref().map(|r| *r).unwrap_or(0.0);

                response.driver = Some(RideDriver {
                    id: driver_id,
                    name,
                    avatar,
                    phone,
                    rating: Some(rating),
                    total_rides: driver.total_rides,
                    vehicle_model: driver.vehicle_model,
                    vehicle_color: driver.vehicle_colour,
                    vehicle_plate: driver.vehicle_plate,
                });
            }
        }

        Ok(response)
    }

    /// Convert string status to enum for check
    fn is_active_status(status: &crate::model::RideStatus) -> bool {
        matches!(
            status,
            crate::model::RideStatus::Requested
                | crate::model::RideStatus::Accepted
                | crate::model::RideStatus::Arrived
                | crate::model::RideStatus::InProgress
                | crate::model::RideStatus::PitStop
                | crate::model::RideStatus::Stopped
        )
    }

    /// Driver accepts a ride request
    pub async fn accept_ride(
        &self,
        ride_id: Uuid,
        driver_id: Uuid,
    ) -> Result<RideResponse, AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        if !matches!(ride.status, crate::model::RideStatus::Requested) {
            return Err(AppError::BadRequest(
                "Can only accept rides in 'requested' status".to_string(),
            ));
        }

        // Verify driver matches the requested driver
        if let Some(req_driver_id) = ride.driver_id {
            if req_driver_id != driver_id {
                return Err(AppError::Unauthorized(
                    "This ride is not assigned to you".to_string(),
                ));
            }
        } else {
            return Err(AppError::BadRequest(
                "Ride has no driver assigned".to_string(),
            ));
        }

        let updated_ride = self
            .repository
            .update_ride_status(ride_id, crate::model::RideStatus::Accepted)
            .await?;

        self.enrich_ride_response(updated_ride).await
    }

    /// Driver cancels an accepted ride
    pub async fn driver_cancel_ride(
        &self,
        ride_id: Uuid,
        driver_id: Uuid,
        reason: String,
    ) -> Result<RideResponse, AppError> {
        self.cancel_ride(ride_id, driver_id, reason, "driver").await
    }

    /// Start a ride
    pub async fn start_ride(
        &self,
        ride_id: Uuid,
        driver_id: Uuid,
    ) -> Result<RideResponse, AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        if ride.driver_id != Some(driver_id) {
            return Err(AppError::Unauthorized("Not authorized".to_string()));
        }

        if !matches!(
            ride.status,
            crate::model::RideStatus::Accepted | crate::model::RideStatus::Arrived
        ) {
            return Err(AppError::BadRequest(
                "Ride must be accepted or arrived to start".to_string(),
            ));
        }

        let updated = self
            .repository
            .update_ride_status(ride_id, crate::model::RideStatus::InProgress)
            .await?;
        self.enrich_ride_response(updated).await
    }

    /// End a ride
    pub async fn end_ride(&self, ride_id: Uuid, driver_id: Uuid) -> Result<RideResponse, AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        if ride.driver_id != Some(driver_id) {
            return Err(AppError::Unauthorized("Not authorized".to_string()));
        }

        if !matches!(
            ride.status,
            crate::model::RideStatus::InProgress
                | crate::model::RideStatus::Stopped
                | crate::model::RideStatus::PitStop
        ) {
            return Err(AppError::BadRequest("Ride is not in progress".to_string()));
        }

        let updated = self
            .repository
            .update_ride_status(ride_id, crate::model::RideStatus::Completed)
            .await?;
        self.enrich_ride_response(updated).await
    }

    /// Driver marks arrival at pickup location
    pub async fn mark_ride_arrived(
        &self,
        ride_id: Uuid,
        driver_id: Uuid,
    ) -> Result<(RideResponse, Uuid), AppError> {
        let ride = self
            .repository
            .get_ride(ride_id)
            .await?
            .ok_or(AppError::NotFound("Ride not found".to_string()))?;

        // Verify driver is assigned to this ride
        if ride.driver_id != Some(driver_id) {
            return Err(AppError::Unauthorized("Not authorized".to_string()));
        }

        // Only allow transition from "accepted" status
        if !matches!(ride.status, crate::model::RideStatus::Accepted) {
            return Err(AppError::BadRequest(
                "Can only mark arrival from 'accepted' status".to_string(),
            ));
        }

        let updated = self
            .repository
            .update_ride_status(ride_id, crate::model::RideStatus::Arrived)
            .await?;

        // Return both the response and the rider_id for notification
        Ok((
            self.enrich_ride_response(updated.clone()).await?,
            updated.rider_id,
        ))
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

        if !matches!(ride.status, crate::model::RideStatus::Completed) {
            return Err(AppError::BadRequest(
                "Can only pay for completed rides".to_string(),
            ));
        }

        let updated_ride = self
            .repository
            .update_ride_status(ride_id, crate::model::RideStatus::Paid)
            .await?;
        self.enrich_ride_response(updated_ride).await
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

        if !matches!(
            ride.status,
            crate::model::RideStatus::Completed | crate::model::RideStatus::Paid
        ) {
            return Err(AppError::BadRequest(
                "Can only rate completed rides".to_string(),
            ));
        }

        // Validate rating
        if req.rating < 1.0 || req.rating > 5.0 {
            return Err(AppError::BadRequest(
                "Rating must be between 1 and 5".to_string(),
            ));
        }

        self.repository
            .update_ride_rating(ride_id, req.rating, req.comment)
            .await?;

        Ok(())
    }

    /// Get current driver location for an active ride
    pub async fn get_driver_location(
        &self,
        ride_id: Uuid,
        rider_id: Uuid,
    ) -> Result<Location, AppError> {
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
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Driver not found".to_string()))?;

        Ok(Location {
            address: "Current Driver Location".to_string(),
            latitude: driver.current_latitude.unwrap_or(0.0),
            longitude: driver.current_longitude.unwrap_or(0.0),
        })
    }
}
