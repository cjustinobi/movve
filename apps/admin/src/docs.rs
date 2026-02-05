use crate::handlers;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::approve_driver_license_image,
        handlers::approve_insurance_image,
        handlers::approve_vehicle_image,
    ),
    components(
        schemas(common::ApiResponse<common::EmptyData>, common::EmptyData)
    ),
    tags(
        (name = "Admin", description = "Admin management endpoints")
    )
)]
pub struct AdminApiDoc;
