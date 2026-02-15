use crate::{handlers, model};
use utoipa::OpenApi;
use common::{ApiResponse, EmptyData};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::update_driver_license_verification,
        handlers::update_insurance_verification,
        handlers::update_vehicle_verification,
        handlers::get_users,
        handlers::toggle_user_suspension,
        handlers::delete_user,
        handlers::get_user_stats,
        handlers::update_user,
        handlers::get_user_details,
        handlers::get_online_driver_locations,
        handlers::get_driver_stats,
    ),
    components(
        schemas(
            ApiResponse<EmptyData>,
            EmptyData,
            model::User,
            model::PaginatedResponse<model::User>, 
            model::SuspendUserRequest,
            model::UpdateProfileRequest,
            model::UserRole,
            model::Gender,
            model::DriverMapLocation,
            model::DriverStatsResponse,
            model::UserStatsResponse,
            model::UpdateVerificationRequest
        )
    ),
    tags(
        (name = "Admin", description = "Admin management endpoints")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub struct AdminApiDoc;
