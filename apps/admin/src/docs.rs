use crate::handlers;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::update_driver_license_verification,
        handlers::update_insurance_verification,
        handlers::update_vehicle_verification,
    ),
    components(
        schemas(common::ApiResponse<common::EmptyData>, common::EmptyData)
    ),
    tags(
        (name = "Admin", description = "Admin management endpoints")
    )
)]
pub struct AdminApiDoc;
