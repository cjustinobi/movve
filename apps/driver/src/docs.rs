use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::create_driver,
        crate::handlers::get_driver,
        crate::handlers::list_drivers,
        crate::handlers::upload_driver_license,
        crate::handlers::upload_vehicle_image,
        crate::handlers::upload_vehicle_insurance,
    ),
    components(schemas(
        crate::model::Driver,
        crate::model::NewDriver,
        crate::model::DriverStatus,
        crate::model::VehicleType,
    )),
    tags(
        (name = "Driver", description = "Endpoints for managing drivers")
    )
)]
pub struct DriverApiDoc;
