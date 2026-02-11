use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::create_driver,
        crate::handlers::list_drivers,
        crate::handlers::get_driver,
        crate::handlers::get_driver_by_user,
        crate::handlers::get_driver_location,
        crate::handlers::get_vehicle_types,
        crate::handlers::update_location,
        crate::handlers::update_location_ws,
        crate::handlers::update_status,
        crate::handlers::upload_driver_license,
        crate::handlers::upload_vehicle_image,
        crate::handlers::upload_vehicle_insurance,
        crate::handlers::start_ride,
        crate::handlers::end_ride,
        crate::handlers::mark_ride_arrived,
        crate::handlers::accept_ride,
        crate::handlers::cancel_ride,
        // crate::handlers::send_message,
        // crate::handlers::get_messages,
    ),
    components(schemas(
        crate::model::Driver,
        crate::model::NewDriver,
        crate::model::DriverStatus,
        crate::model::VehicleType,
        crate::model::UpdateStatusRequest,
        crate::model::DriverLocation,
    )),
    tags(
        (name = "Driver", description = "Endpoints for managing drivers")
    )
)]
pub struct DriverApiDoc;
