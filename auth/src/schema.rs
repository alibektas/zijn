// @generated automatically by Diesel CLI.

diesel::table! {
    active_sessions (session_id) {
        id -> Uuid,
        session_id -> Uuid,
        expires_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        password -> Text,
    }
}

diesel::joinable!(active_sessions -> users (id));

diesel::allow_tables_to_appear_in_same_query!(
    active_sessions,
    users,
);
