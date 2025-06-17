#[cfg(test)]
use crate::{db::DbConnection, schema::arepl::submissions};

#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};

#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_submission(
    level_id: Uuid,
    user_id: Uuid,
    conn: &mut DbConnection,
) -> Uuid {
    diesel::insert_into(submissions::table)
        .values((
            submissions::level_id.eq(level_id),
            submissions::submitted_by.eq(user_id),
            submissions::mobile.eq(false),
            submissions::video_url.eq("https://video.com"),
            submissions::raw_url.eq("https://raw.com"),
            submissions::priority.eq(false),
            submissions::user_notes.eq("Test submission"),
            submissions::completion_time.eq(1000000),
            submissions::mod_menu.eq("Mega hack"),
        ))
        .returning(submissions::id)
        .get_result::<Uuid>(conn)
        .expect("Failed to create test submission!")
}
