#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use crate::app_data::db::DbAppState;
#[cfg(test)]
use crate::aredl::submissions::SubmissionStatus;
#[cfg(test)]
use crate::schema::aredl::{records, submissions};
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_record(db: &Arc<DbAppState>, user_id: Uuid, level_id: Uuid) -> Uuid {
    let conn = &mut db.connection().unwrap();
    let submission_id = diesel::insert_into(submissions::table)
        .values((
            submissions::submitted_by.eq(user_id),
            submissions::video_url.eq("https://example.com/video.mp4"),
            submissions::level_id.eq(level_id),
            submissions::status.eq(SubmissionStatus::Accepted),
            submissions::mobile.eq(false),
        ))
        .returning(submissions::id)
        .get_result::<Uuid>(conn)
        .expect("Failed to create test aredl record");

    return records::table
        .filter(records::submission_id.eq(submission_id))
        .select(records::id)
        .first::<Uuid>(conn)
        .expect("Failed to retrieve test aredl record ID");
}
