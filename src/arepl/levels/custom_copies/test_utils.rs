#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use super::{LevelCustomCopyStatus, LevelCustomCopyType};
#[cfg(test)]
use crate::app_data::db::DbAppState;
#[cfg(test)]
use crate::schema::arepl::level_custom_copies;
#[cfg(test)]
use diesel::{ExpressionMethods as _, RunQueryDsl as _};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_custom_copy(db: &Arc<DbAppState>, level_id: Uuid, user: Uuid) -> Uuid {
    let copy_id = rand::random_range(1..=100_000_000);
    let level_uuid = Uuid::new_v4();

    diesel::insert_into(level_custom_copies::table)
        .values((
            level_custom_copies::id.eq(level_uuid),
            level_custom_copies::added_by.eq(user),
            level_custom_copies::level_id.eq(level_id),
            level_custom_copies::copy_id.eq(copy_id),
            level_custom_copies::description.eq("Test"),
            level_custom_copies::status.eq(LevelCustomCopyStatus::Allowed),
            level_custom_copies::id_type.eq(LevelCustomCopyType::Bugfix),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test custom copy id");

    level_uuid
}
