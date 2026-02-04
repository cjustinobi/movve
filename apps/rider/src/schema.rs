// @generated automatically by Diesel CLI.

diesel::table! {
    conversations (id) {
        id -> Uuid,
        #[max_length = 50]
        context_type -> Varchar,
        context_id -> Uuid,
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
    rides (id) {
        id -> Uuid,
        rider_id -> Uuid,
        driver_id -> Nullable<Uuid>,
        pickup -> Jsonb,
        destination -> Jsonb,
        #[max_length = 50]
        status -> Varchar,
        fare -> Float8,
        distance -> Float8,
        duration -> Float8,
        #[max_length = 10]
        otp -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(messages -> conversations (conversation_id));

diesel::allow_tables_to_appear_in_same_query!(conversations, messages, rides,);
