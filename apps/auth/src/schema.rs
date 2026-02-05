// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "gender"))]
    pub struct Gender;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;
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
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        profile_completed -> Bool,
    }
}

diesel::joinable!(email_verification_tokens -> users (user_id));
diesel::joinable!(password_resets -> users (user_id));
diesel::joinable!(refresh_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    email_verification_tokens,
    password_resets,
    refresh_tokens,
    users,
);
