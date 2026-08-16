#[cfg(test)]
use {
    super::LevelNotesType, crate::app_data::db::DbAppState, crate::schema::aredl::level_notes,
    diesel::prelude::*, std::sync::Arc, uuid::Uuid,
};

#[cfg(test)]
pub async fn create_test_note(db: &Arc<DbAppState>, level_id: Uuid, user: Uuid) -> Uuid {
    let level_uuid = Uuid::new_v4();

    diesel::insert_into(level_notes::table)
        .values((
            level_notes::id.eq(level_uuid),
            level_notes::added_by.eq(user),
            level_notes::level_id.eq(level_id),
            level_notes::note_type.eq(LevelNotesType::PublicNotes),
            level_notes::note.eq("This is a test note".to_owned()),
            level_notes::timestamp.eq(chrono::Utc::now()),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test note");

    level_uuid
}
