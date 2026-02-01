// @generated automatically by Diesel CLI.

diesel::table! {
    rides (id) {
        id -> Uuid,
        rider_id -> Uuid,
        driver_id -> Nullable<Uuid>,
        #[sql_name = "pickup"]
        pickup -> Jsonb,
        #[sql_name = "destination"]
        destination -> Jsonb,
        status -> Varchar,
        fare -> Float8,
        distance -> Float8,   // Distance in meters
        duration -> Float8,   // Duration in seconds
        otp -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
