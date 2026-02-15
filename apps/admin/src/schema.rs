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
    #[diesel(postgres_type(name = "gender"))]
    pub struct Gender;

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

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Gender;
    use super::sql_types::UserRole;

    users (id) {
        id -> Uuid,
        #[max_length = 100]
        first_name -> Nullable<Varchar>,
        #[max_length = 100]
        last_name -> Nullable<Varchar>,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 100]
        phone -> Nullable<Varchar>,
        gender -> Nullable<Gender>,
        #[max_length = 255]
        avatar -> Nullable<Varchar>,
        #[max_length = 100]
        nok_name -> Nullable<Varchar>,
        #[max_length = 100]
        nok_phone -> Nullable<Varchar>,
        dob -> Nullable<Date>,
        #[max_length = 255]
        password_hash -> Varchar,
        role -> UserRole,
        email_verified -> Bool,
        profile_completed -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        suspended -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(drivers, users,);
