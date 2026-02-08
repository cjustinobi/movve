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
        crate::handlers::update_status,
        crate::handlers::upload_driver_license,
        crate::handlers::upload_vehicle_image,
        crate::handlers::upload_vehicle_insurance,
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
