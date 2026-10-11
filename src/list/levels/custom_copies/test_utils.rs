#[cfg(test)]
use {
    super::{LevelCustomCopyStatus, LevelCustomCopyType},
    crate::{app_data::db::DbAppState, list::List, schema::level_custom_copies},
    diesel::prelude::*,
    std::sync::Arc,
    uuid::Uuid,
};

#[cfg(test)]
pub async fn create_test_custom_copy(
    db: &Arc<DbAppState>,
    list: List,
    level_id: Uuid,
    user: Uuid,
) -> Uuid {
    let copy_id = rand::random_range(1..=100_000_000);
    let copy_uuid = Uuid::new_v4();

    diesel::insert_into(level_custom_copies::table)
        .values((
            level_custom_copies::list_id.eq(list),
            level_custom_copies::id.eq(copy_uuid),
            level_custom_copies::added_by.eq(user),
            level_custom_copies::level_id.eq(level_id),
            level_custom_copies::copy_id.eq(copy_id),
            level_custom_copies::description.eq("Test"),
            level_custom_copies::status.eq(LevelCustomCopyStatus::Allowed),
            level_custom_copies::id_type.eq(LevelCustomCopyType::Bugfix),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test custom copy id");

    copy_uuid
}
