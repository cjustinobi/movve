// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "driver_status"))]
    pub struct DriverStatus;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "gender"))]
    pub struct Gender;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ride_status"))]
    pub struct RideStatus;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vehicle_colour"))]
    pub struct VehicleColour;
}

diesel::table! {
    conversations (id) {
        id -> Uuid,
        #[max_length = 50]
        context_type -> Varchar,
        context_id -> Int8,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
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
        #[max_length = 50]
        vehicle_type -> Varchar,
        vehicle_colour -> VehicleColour,
        #[max_length = 20]
        vehicle_plate -> Varchar,
        #[max_length = 100]
        vehicle_model -> Varchar,
        vehicle_year -> Int4,
        status -> DriverStatus,
        verified -> Bool,
        suspended -> Bool,
        vehicle_verification_completed -> Bool,
        vehicle_capacity -> Int4,
        driver_license_verified -> Bool,
        insurance_verified -> Bool,
        vehicle_image_verified -> Bool,
        rating -> Nullable<Numeric>,
        total_rides -> Nullable<Int4>,
        current_latitude -> Nullable<Numeric>,
        current_longitude -> Nullable<Numeric>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    email_verification_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 10]
        code -> Varchar,
        expires_at -> Timestamptz,
        used -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    messages (id) {
        id -> Uuid,
        conversation_id -> Uuid,
        sender_id -> Uuid,
        #[max_length = 50]
        sender_role -> Varchar,
        content -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    password_resets (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        token -> Varchar,
        expires_at -> Timestamptz,
        used -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    ratings (id) {
        id -> Uuid,
        user_id -> Uuid,
        rater_id -> Uuid,
        rating -> Float8,
        comment -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        token -> Varchar,
        expires_at -> Timestamptz,
        revoked -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RideStatus;

    rides (id) {
        id -> Int8,
        rider_id -> Uuid,
        driver_id -> Nullable<Uuid>,
        pickup -> Jsonb,
        destination -> Jsonb,
        status -> RideStatus,
        fare -> Float8,
        distance -> Float8,
        duration -> Float8,
        #[max_length = 10]
        otp -> Nullable<Varchar>,
        #[max_length = 255]
        cancellation_reason -> Nullable<Varchar>,
        #[max_length = 50]
        cancelled_by -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
        suspended -> Bool,
        email_verified -> Bool,
        profile_completed -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    vehicle_types (id) {
        id -> Uuid,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 100]
        display_name -> Varchar,
        description -> Text,
        base_price -> Float8,
        is_active -> Bool,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(email_verification_tokens -> users (user_id));
diesel::joinable!(messages -> conversations (conversation_id));
diesel::joinable!(password_resets -> users (user_id));
diesel::joinable!(ratings -> users (user_id));
diesel::joinable!(refresh_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    conversations,
    drivers,
    email_verification_tokens,
    messages,
    password_resets,
    ratings,
    refresh_tokens,
    rides,
    users,
    vehicle_types,
);
