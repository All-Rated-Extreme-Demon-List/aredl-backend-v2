use crate::schema::{roles, user_roles};

diesel::table! {
    role_permissions_full (role_id, permission) {
        role_id -> Int4,
        permission -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(role_permissions_full, roles,);
diesel::allow_tables_to_appear_in_same_query!(role_permissions_full, user_roles,);
