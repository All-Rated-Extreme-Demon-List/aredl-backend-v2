use crate::app_data::db::DbConnection;
use crate::auth::Permission;
use crate::schema::permissions;
use crate::schema::{roles, user_roles, users};

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_user(
    conn: &mut DbConnection,
    required_permission: Option<Permission>,
) -> (Uuid, String) {
    let user_id = Uuid::new_v4();
    let username = format!("test_user_{}", user_id);

    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_id),
            users::username.eq(&username),
            users::global_name.eq(&username),
            users::discord_id.eq(None::<String>),
            users::placeholder.eq(false),
            users::country.eq(None::<i32>),
            users::discord_avatar.eq(None::<String>),
            users::discord_banner.eq(None::<String>),
            users::discord_accent_color.eq(None::<i32>),
        ))
        .execute(conn)
        .expect("Failed to create fake user");

    if required_permission.is_some() {
        let privilege_level = permissions::table
            .filter(permissions::permission.eq(required_permission.unwrap().to_string()))
            .select(permissions::privilege_level)
            .first::<i32>(conn)
            .expect("Failed to get privilege level");

        let role_id: i32 = diesel::insert_into(roles::table)
            .values((
                roles::privilege_level.eq(privilege_level),
                roles::role_desc.eq(format!("Test Role - {}", privilege_level)),
            ))
            .returning(roles::id)
            .get_result(conn)
            .expect("Failed to create test role");

        diesel::insert_into(user_roles::table)
            .values((
                user_roles::role_id.eq(role_id),
                user_roles::user_id.eq(user_id),
            ))
            .execute(conn)
            .expect("Failed to assign role to user");
    }

    (user_id, username)
}

#[cfg(test)]
pub async fn create_test_placeholder_user(
    conn: &mut DbConnection,
    required_permission: Option<Permission>,
) -> (Uuid, String) {
    let user_id = Uuid::new_v4();
    let username = format!("test_user_{}", user_id);

    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_id),
            users::username.eq(&username),
            users::global_name.eq(&username),
            users::discord_id.eq(None::<String>),
            users::placeholder.eq(true),
            users::country.eq(None::<i32>),
            users::discord_avatar.eq(None::<String>),
            users::discord_banner.eq(None::<String>),
            users::discord_accent_color.eq(None::<i32>),
        ))
        .execute(conn)
        .expect("Failed to create fake user");

    if required_permission.is_some() {
        let privilege_level = permissions::table
            .filter(permissions::permission.eq(required_permission.unwrap().to_string()))
            .select(permissions::privilege_level)
            .first::<i32>(conn)
            .expect("Failed to get privilege level");

        let role_id: i32 = diesel::insert_into(roles::table)
            .values((
                roles::privilege_level.eq(privilege_level),
                roles::role_desc.eq(format!("Test Role - {}", privilege_level)),
            ))
            .returning(roles::id)
            .get_result(conn)
            .expect("Failed to create test role");

        diesel::insert_into(user_roles::table)
            .values((
                user_roles::role_id.eq(role_id),
                user_roles::user_id.eq(user_id),
            ))
            .execute(conn)
            .expect("Failed to assign role to user");
    }

    (user_id, username)
}
