#[cfg(test)]
use crate::{app_data::db::DbConnection, schema::aredl::submissions};
use crate::{
    aredl::submissions::{history::SubmissionHistory, SubmissionStatus},
    schema::aredl::submission_history,
};

use chrono::Utc;
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
            submissions::mod_menu.eq("Mega hack"),
        ))
        .returning(submissions::id)
        .get_result::<Uuid>(conn)
        .expect("Failed to create test submission!")
}

pub async fn insert_history_entry(
    submission_id: Uuid,
    reviewer_id: Option<Uuid>,
    status: SubmissionStatus,
    conn: &mut DbConnection,
) {
    let history = SubmissionHistory {
        id: Uuid::new_v4(),
        submission_id,
        reviewer_notes: None,
        status,
        timestamp: Utc::now(),
        user_notes: None,
        reviewer_id,
    };
    diesel::insert_into(submission_history::table)
        .values(&history)
        .execute(conn)
        .expect("Failed to insert submission history");
}
