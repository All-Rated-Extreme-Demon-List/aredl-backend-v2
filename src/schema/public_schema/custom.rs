use crate::schema::{clan_invites, merge_requests, permissions, roles, user_roles, users};

diesel::table! {
    role_permissions_full (role_id, permission) {
        role_id -> Int4,
        permission -> Varchar,
    }
}

diesel::joinable!(clan_invites -> users (user_id));
diesel::joinable!(merge_requests -> users (primary_user));

diesel::joinable!(role_permissions_full -> roles (role_id));
diesel::joinable!(role_permissions_full -> permissions (permission));

diesel::allow_tables_to_appear_in_same_query!(role_permissions_full, roles,);

diesel::allow_tables_to_appear_in_same_query!(role_permissions_full, permissions,);

diesel::allow_tables_to_appear_in_same_query!(role_permissions_full, user_roles);
