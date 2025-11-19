#[cfg(test)]
use crate::app_data::db::DbConnection;
#[cfg(test)]
use crate::arepl::submissions::SubmissionStatus;
#[cfg(test)]
use crate::schema::arepl::{records, submissions};
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_record(conn: &mut DbConnection, user_id: Uuid, level_id: Uuid) -> Uuid {
    let (level_id, submitted_by) = diesel::insert_into(submissions::table)
        .values((
            submissions::submitted_by.eq(user_id),
            submissions::video_url.eq("https://example.com/video.mp4"),
            submissions::level_id.eq(level_id),
            submissions::status.eq(SubmissionStatus::Accepted),
            submissions::mobile.eq(false),
        ))
        .returning((submissions::level_id, submissions::submitted_by))
        .get_result::<(Uuid, Uuid)>(conn)
        .expect("Failed to create test aredl record");

    return records::table
        .filter(records::level_id.eq(level_id))
        .filter(records::submitted_by.eq(submitted_by))
        .select(records::id)
        .first::<Uuid>(conn)
        .expect("Failed to retrieve test aredl record ID");
}
