#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use crate::arepl::levels::{Level, LevelStatus};
#[cfg(test)]
use crate::schema::arepl::{levels, levels_created, pack_levels, position_history};
#[cfg(test)]
use crate::users::test_utils::create_test_user;
#[cfg(test)]
use crate::{app_data::db::DbAppState, arepl::records::test_utils::create_test_record};
#[cfg(test)]
use diesel::{
    sql_query, ExpressionMethods as _, QueryDsl as _, RunQueryDsl as _, SelectableHelper as _,
};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_level(db: &Arc<DbAppState>) -> Uuid {
    let level_id = rand::random_range(1..=100_000_000);
    let level_uuid = Uuid::new_v4();
    let publisher = create_test_user(db, None).await.0;

    diesel::insert_into(levels::table)
        .values((
            levels::id.eq(level_uuid),
            levels::position.eq(1),
            levels::name.eq(format!("Test Level {level_id}")),
            levels::publisher_id.eq(publisher),
            levels::status.eq(LevelStatus::MainList),
            levels::requires_raw_footage.eq(false),
            levels::level_id.eq(level_id),
            levels::two_player.eq(false),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test arepl level");

    level_uuid
}

#[cfg(test)]
pub async fn create_test_level_with_publisher(db: &Arc<DbAppState>, publisher: Uuid) -> Uuid {
    let level_id = create_test_level(db).await;

    diesel::update(levels::table)
        .filter(levels::id.eq(level_id))
        .set(levels::publisher_id.eq(publisher))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to update test arepl level publisher");

    level_id
}

#[cfg(test)]
pub async fn add_test_level_creators(db: &Arc<DbAppState>, level_id: Uuid, creators: &[Uuid]) {
    diesel::insert_into(levels_created::table)
        .values(
            creators
                .iter()
                .map(|creator| {
                    (
                        levels_created::level_id.eq(level_id),
                        levels_created::user_id.eq(*creator),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create arepl level creators");
}

#[cfg(test)]
pub async fn set_test_level_status(
    db: &Arc<DbAppState>,
    level_id: Uuid,
    status: LevelStatus,
    position: Option<i32>,
) {
    diesel::update(levels::table.filter(levels::id.eq(level_id)))
        .set((levels::status.eq(status), levels::position.eq(position)))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to update test arepl level status");
}

#[cfg(test)]
pub async fn set_test_level_position(db: &Arc<DbAppState>, level_id: Uuid, position: Option<i32>) {
    diesel::update(levels::table.filter(levels::id.eq(level_id)))
        .set(levels::position.eq(position))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to update test arepl level position");
}

#[cfg(test)]
pub async fn create_test_level_with_record(db: &Arc<DbAppState>, user_id: Uuid) -> (Uuid, Uuid) {
    let level_id = create_test_level(db).await;
    let record_id = create_test_record(db, user_id, level_id).await;
    (level_id, record_id)
}

#[cfg(test)]
pub async fn get_test_level(db: &Arc<DbAppState>, level_id: Uuid) -> Level {
    levels::table
        .filter(levels::id.eq(level_id))
        .select(Level::as_select())
        .first::<Level>(&mut db.connection().unwrap())
        .expect("Failed to get test arepl level")
}

#[cfg(test)]
pub async fn set_test_level_gd_id(db: &Arc<DbAppState>, level_id: Uuid, gd_id: i32) {
    diesel::update(levels::table.filter(levels::id.eq(level_id)))
        .set(levels::level_id.eq(gd_id))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test arepl level GD ID");
}

#[cfg(test)]
pub async fn refresh_test_position_history(db: &Arc<DbAppState>) {
    sql_query("REFRESH MATERIALIZED VIEW arepl.position_history_full_view")
        .execute(&mut db.connection().unwrap())
        .expect("Failed to refresh position history view");
}

#[cfg(test)]
pub fn latest_test_position_history_created_at(
    db: &Arc<DbAppState>,
    level_id: Uuid,
) -> chrono::DateTime<chrono::Utc> {
    position_history::table
        .filter(position_history::affected_level.eq(level_id))
        .order_by(position_history::i.desc())
        .select(position_history::created_at)
        .first(&mut db.connection().unwrap())
        .expect("Failed to fetch test arepl level position history timestamp")
}

#[cfg(test)]
pub fn add_test_level_to_pack(db: &Arc<DbAppState>, level_id: Uuid, pack_id: Uuid) {
    diesel::insert_into(pack_levels::table)
        .values((
            pack_levels::pack_id.eq(pack_id),
            pack_levels::level_id.eq(level_id),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to add test arepl level to pack");
}
