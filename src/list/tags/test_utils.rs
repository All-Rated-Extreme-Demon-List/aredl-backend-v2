#[cfg(test)]
use {
    super::TagCategory,
    crate::{
        app_data::db::DbAppState,
        list::List,
        schema::{level_tags, tags},
    },
    diesel::prelude::*,
    std::sync::Arc,
    uuid::Uuid,
};

#[cfg(test)]
pub async fn create_test_tag(db: &Arc<DbAppState>) -> Uuid {
    let tag_id = Uuid::new_v4();

    diesel::insert_into(tags::table)
        .values((
            tags::id.eq(tag_id),
            tags::list_id.eq(List::Classic),
            tags::name.eq("Wave"),
            tags::description.eq("Extreme demons that feature mostly wave gameplay."),
            tags::category.eq(TagCategory::Gamemode),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test tag");

    tag_id
}

#[cfg(test)]
pub async fn create_test_level_tag(db: &Arc<DbAppState>, level_id: Uuid, tag_id: Uuid) {
    diesel::insert_into(level_tags::table)
        .values((
            level_tags::list_id.eq(List::Classic),
            level_tags::level_id.eq(level_id),
            level_tags::tag_id.eq(tag_id),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test level tag");
}
