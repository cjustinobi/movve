use crate::handlers::*;
use crate::model::*;
use utoipa::{
    OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        estimate_ride,
        create_ride,
        get_ride,
        get_rides,
        cancel_ride,
        pay_ride,
        rate_driver,
        get_driver_location,
        get_ride_status,
    ),
    components(
        schemas(
            RideResponse,
            CreateRideRequest,
            RideEstimateRequest,
            RideEstimateResponse,
            PayRideRequest,
            RateDriverRequest,
            DriverOption,
        )
    ),
    tags(
        (name = "Rider", description = "Rider management endpoints")
    ),
)]
pub struct RiderApiDoc;

