#[cfg(test)]
use crate::{
    db::DbConnection,
    schema::{roles, user_roles},
};
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_role(conn: &mut DbConnection, privilege_level: i32) -> i32 {
    let role_name = format!("Test Role {}", privilege_level);
    diesel::insert_into(roles::table)
        .values((
            roles::role_desc.eq(role_name),
            roles::privilege_level.eq(privilege_level),
        ))
        .returning(roles::id)
        .get_result::<i32>(conn)
        .expect("Failed to create test role!")
}

#[cfg(test)]
pub async fn create_test_role_with_user(
    conn: &mut DbConnection,
    privilege_level: i32,
) -> (i32, Uuid) {
    use crate::users::test_utils::create_test_user;

    let role_id = create_test_role(conn, privilege_level).await;
    let (user_id, _) = create_test_user(conn, None).await;
    diesel::insert_into(user_roles::table)
        .values((
            user_roles::user_id.eq(user_id),
            user_roles::role_id.eq(role_id),
        ))
        .execute(conn)
        .expect("Failed to assign role to user!");
    (role_id, user_id)
}

#[cfg(test)]
pub async fn add_user_to_role(conn: &mut DbConnection, role_id: i32, user_id: Uuid) {
    diesel::insert_into(user_roles::table)
        .values((
            user_roles::user_id.eq(user_id),
            user_roles::role_id.eq(role_id),
        ))
        .execute(conn)
        .expect("Failed to assign role to user!");
}
