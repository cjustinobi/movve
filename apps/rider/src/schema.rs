// @generated automatically by Diesel CLI.

diesel::table! {
    rides (id) {
        id -> Uuid,
        rider_id -> Uuid,
        driver_id -> Nullable<Uuid>,
        pickup -> Text,
        destination -> Text,
        status -> Text,
        fare -> Float8,
        otp -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
