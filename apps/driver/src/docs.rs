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
    )),
    tags(
        (name = "Driver", description = "Driver service endpoints")
    ),
    info(
        title = "Driver Service API",
        version = "1.0.0",
        description = "Movve Driver service"
    )
)]
pub struct DriverApiDoc;
