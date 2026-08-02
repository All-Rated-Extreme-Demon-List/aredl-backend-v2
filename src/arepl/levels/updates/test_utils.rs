#[cfg(test)]
use {
    super::LevelUpdateType, crate::app_data::db::DbAppState, crate::schema::arepl::level_updates,
    diesel::prelude::*, std::sync::Arc, uuid::Uuid,
};

#[cfg(test)]
pub async fn create_test_update(db: &Arc<DbAppState>, level_id: Uuid) -> Uuid {
    let update_uuid = Uuid::new_v4();

    diesel::insert_into(level_updates::table)
        .values((
            level_updates::id.eq(update_uuid),
            level_updates::level_id.eq(level_id),
            level_updates::changelog.eq("Test update"),
            level_updates::update_type.eq(LevelUpdateType::Buff),
            level_updates::timestamp.eq(chrono::Utc::now()),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test update");

    update_uuid
}
