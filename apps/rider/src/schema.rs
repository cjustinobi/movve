// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ride_status"))]
    pub struct RideStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RideStatus;

    conversations (id) {
        id -> Uuid,
        #[max_length = 50]
        context_type -> Varchar,
        context_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RideStatus;

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
    use diesel::sql_types::*;
    use super::sql_types::RideStatus;

    rides (id) {
        id -> Uuid,
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
        cancellation_reason -> Nullable<Varchar>,
        cancelled_by -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(messages -> conversations (conversation_id));

diesel::allow_tables_to_appear_in_same_query!(conversations, messages, rides,);
