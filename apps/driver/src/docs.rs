use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::create_driver,
        crate::handlers::get_driver,
        crate::handlers::list_drivers,
    ),
    components(schemas(
        crate::model::Driver,
        crate::model::NewDriver,
        crate::model::DriverStatus,
        crate::model::VehicleType,
    )),
    tags(
        (name = "Driver Service", description = "Endpoints for managing drivers")
    )
)]
pub struct DriverApiDoc;
