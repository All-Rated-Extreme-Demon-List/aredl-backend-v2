#[cfg(test)]
use crate::arepl::records::test_utils::create_test_record;
#[cfg(test)]
use crate::db::DbConnection;
#[cfg(test)]
use crate::schema::arepl::levels;
#[cfg(test)]
use crate::users::test_utils::create_test_user;
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
#[cfg(test)]
use rand::Rng;
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_level(conn: &mut DbConnection) -> Uuid {
    let mut rng = rand::rng();
    let level_id = rng.random_range(1..=100000000);
    let level_uuid = Uuid::new_v4();
    let publisher = create_test_user(conn, None).await.0;

    diesel::insert_into(levels::table)
        .values((
            levels::id.eq(level_uuid),
            levels::position.eq(1),
            levels::name.eq(format!("Test Level {}", level_id)),
            levels::publisher_id.eq(publisher),
            levels::legacy.eq(false),
            levels::level_id.eq(level_id),
            levels::two_player.eq(false),
        ))
        .execute(conn)
        .expect("Failed to create test aredl level");

    level_uuid
}

#[cfg(test)]
pub async fn create_test_level_with_record(conn: &mut DbConnection, user_id: Uuid) -> (Uuid, Uuid) {
    let level_id = create_test_level(conn).await;
    let record_id = create_test_record(conn, user_id, level_id).await;
    (level_id, record_id)
}
