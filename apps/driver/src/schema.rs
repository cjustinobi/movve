// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "driver_status"))]
    pub struct DriverStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vehicle_colour"))]
    pub struct VehicleColour;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vehicle_type"))]
    pub struct VehicleType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::VehicleType;
    use super::sql_types::VehicleColour;
    use super::sql_types::DriverStatus;

    drivers (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 50]
        license_number -> Varchar,
        #[max_length = 255]
        driver_license_image -> Varchar,
        #[max_length = 50]
        insurance_number -> Nullable<Varchar>,
        #[max_length = 255]
        insurance_image -> Nullable<Varchar>,
        #[max_length = 255]
        vehicle_image -> Varchar,
        vehicle_type -> VehicleType,
        vehicle_colour -> VehicleColour,
        #[max_length = 20]
        vehicle_plate -> Varchar,
        #[max_length = 100]
        vehicle_model -> Varchar,
        vehicle_year -> Int4,
        status -> DriverStatus,
        verified -> Bool,
        suspended -> Bool,
        rating -> Nullable<Numeric>,
        total_rides -> Nullable<Int4>,
        current_latitude -> Nullable<Numeric>,
        current_longitude -> Nullable<Numeric>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        vehicle_verification_completed -> Bool,
        driver_license_verified -> Bool,
        insurance_verified -> Bool,
        vehicle_image_verified -> Bool,
        vehicle_capacity -> Int4,
    }
}
