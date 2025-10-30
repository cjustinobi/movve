// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "driver_status"))]
    pub struct DriverStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vehicle_type"))]
    pub struct VehicleType;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::VehicleType;
    use super::sql_types::DriverStatus;

    drivers (id) {
        id -> Uuid,
        user_id -> Uuid,
        phone -> Varchar,
        license_number -> Varchar,
        vehicle_type -> VehicleType,
        vehicle_plate -> Varchar,
        vehicle_model -> Varchar,
        vehicle_year -> Int4,
        status -> DriverStatus,
        rating -> Nullable<Numeric>,
        total_rides -> Nullable<Int4>,
        current_latitude -> Nullable<Numeric>,
        current_longitude -> Nullable<Numeric>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}