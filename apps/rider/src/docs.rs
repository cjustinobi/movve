use crate::model::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::estimate_ride,
        crate::handlers::create_ride,
        crate::handlers::get_ride,
        crate::handlers::get_rides,
        crate::handlers::cancel_ride,
        crate::handlers::pay_ride,
        crate::handlers::rate_driver,
        crate::handlers::get_driver_location,
        crate::handlers::get_ride_status,
        crate::handlers::send_message,
        crate::handlers::get_messages,
        crate::handlers::get_chat_context_types,
    ),
    components(
        schemas(
            RideResponse,
            CreateRideRequest,
            CancelRideRequest,
            RideEstimateRequest,
            RideEstimateResponse,
            PayRideRequest,
            RateDriverRequest,
            DriverOption,
            SendMessageRequest,
            MessageResponse,
            Location,
        )
    ),
    tags(
        (name = "Rider", description = "Rider management endpoints")
    ),
)]
pub struct RiderApiDoc;
