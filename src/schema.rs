// @generated automatically by Diesel CLI.

diesel::table! {
    aredl_levels (id) {
        id -> Uuid,
        position -> Int4,
        name -> Varchar,
        points -> Int4,
        legacy -> Bool,
        level_id -> Int4,
        two_player -> Bool,
    }
}

diesel::table! {
    aredl_position_history (i) {
        i -> Int4,
        new_position -> Nullable<Int4>,
        old_position -> Nullable<Int4>,
        legacy -> Nullable<Bool>,
        affected_level -> Uuid,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    oauth_requests (csrf_state) {
        csrf_state -> Varchar,
        pkce_verifier -> Varchar,
        nonce -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    permissions (permission) {
        privilege_level -> Int4,
        permission -> Varchar,
    }
}

diesel::table! {
    roles (id) {
        id -> Int4,
        privilege_level -> Int4,
        role_desc -> Varchar,
    }
}

diesel::table! {
    user_roles (role_id, user_id) {
        role_id -> Int4,
        user_id -> Uuid,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        global_name -> Varchar,
        discord_id -> Nullable<Varchar>,
        placeholder -> Bool,
        discord_avatar -> Nullable<Varchar>,
        discord_banner -> Nullable<Varchar>,
        discord_accent_color -> Nullable<Int4>,
    }
}

diesel::joinable!(aredl_position_history -> aredl_levels (affected_level));
diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(user_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    aredl_levels,
    aredl_position_history,
    oauth_requests,
    permissions,
    roles,
    user_roles,
    users,
);
