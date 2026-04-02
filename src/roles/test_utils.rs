#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use crate::{
    app_data::db::DbAppState,
    schema::{roles, user_roles},
};
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_role(db: &Arc<DbAppState>, privilege_level: i32) -> i32 {
    let role_name = format!("Test Role {}", privilege_level);
    create_test_role_with_desc(db, privilege_level, &role_name).await
}

#[cfg(test)]
pub async fn create_test_role_with_desc(
    db: &Arc<DbAppState>,
    privilege_level: i32,
    role_desc: &str,
) -> i32 {
    diesel::insert_into(roles::table)
        .values((
            roles::role_desc.eq(role_desc),
            roles::privilege_level.eq(privilege_level),
        ))
        .returning(roles::id)
        .get_result::<i32>(&mut db.connection().unwrap())
        .expect("Failed to create test role!")
}

#[cfg(test)]
pub async fn create_test_hidden_role(db: &Arc<DbAppState>, privilege_level: i32) -> i32 {
    let role_id = create_test_role(db, privilege_level).await;

    diesel::update(roles::table.filter(roles::id.eq(role_id)))
        .set(roles::hide.eq(true))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to hide test role!");

    role_id
}

#[cfg(test)]
pub async fn create_test_role_with_user(db: &Arc<DbAppState>, privilege_level: i32) -> (i32, Uuid) {
    use crate::users::test_utils::create_test_user;

    let role_id = create_test_role(db, privilege_level).await;
    let (user_id, _) = create_test_user(db, None).await;
    diesel::insert_into(user_roles::table)
        .values((
            user_roles::user_id.eq(user_id),
            user_roles::role_id.eq(role_id),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to assign role to user!");
    (role_id, user_id)
}

#[cfg(test)]
pub async fn create_test_hidden_role_with_user(
    db: &Arc<DbAppState>,
    privilege_level: i32,
) -> (i32, Uuid) {
    use crate::users::test_utils::create_test_user;

    let role_id = create_test_hidden_role(db, privilege_level).await;
    let (user_id, _) = create_test_user(db, None).await;
    add_user_to_role(db, role_id, user_id).await;
    (role_id, user_id)
}

#[cfg(test)]
pub async fn add_user_to_role(db: &Arc<DbAppState>, role_id: i32, user_id: Uuid) {
    diesel::insert_into(user_roles::table)
        .values((
            user_roles::user_id.eq(user_id),
            user_roles::role_id.eq(role_id),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to assign role to user!");
}
