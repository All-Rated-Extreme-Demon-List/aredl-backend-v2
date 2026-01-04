#[cfg(test)]
use {
    crate::{
        app_data::db::DbAppState,
        aredl::{
            levels::test_utils::create_test_level,
            submissions::{history::SubmissionHistory, SubmissionStatus},
        },
        schema::aredl::{submission_history, submissions},
    },
    chrono::{DateTime, Utc},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    std::sync::Arc,
    uuid::Uuid,
};

#[cfg(test)]
pub async fn create_test_submission(level_id: Uuid, user_id: Uuid, db: &Arc<DbAppState>) -> Uuid {
    diesel::insert_into(submissions::table)
        .values((
            submissions::level_id.eq(level_id),
            submissions::submitted_by.eq(user_id),
            submissions::mobile.eq(false),
            submissions::video_url.eq("https://youtube.com/watch?v=xvFZjo5PgG0"),
            submissions::raw_url.eq("https://www.youtube.com/watch?v=xvFZjo5PgG0"),
            submissions::priority.eq(false),
            submissions::user_notes.eq("Test submission"),
            submissions::mod_menu.eq("Mega hack"),
        ))
        .returning(submissions::id)
        .get_result::<Uuid>(&mut db.connection().unwrap())
        .expect("Failed to create test submission!")
}

pub async fn insert_history_entry(
    submission_id: Uuid,
    reviewer_id: Option<Uuid>,
    status: SubmissionStatus,
    db: &Arc<DbAppState>,
) {
    let history = SubmissionHistory {
        id: Uuid::new_v4(),
        submission_id,
        reviewer_notes: None,
        video_url: Some("https://video.com".to_string()),
        raw_url: Some("https://raw.com".to_string()),
        mobile: Some(false),
        ldm_id: None,
        mod_menu: Some("Mega Hack v8".to_string()),
        status,
        timestamp: Utc::now(),
        user_notes: None,
        private_reviewer_notes: None,
        reviewer_id,
    };
    diesel::insert_into(submission_history::table)
        .values(&history)
        .execute(&mut db.connection().unwrap())
        .expect("Failed to insert submission history");
}

#[cfg(test)]
pub async fn create_two_test_submissions_with_different_timestamps(
    db: &Arc<DbAppState>,
    submitter_id: Uuid,
) -> (Uuid, Uuid) {
    let level_a = create_test_level(db).await;
    let level_b = create_test_level(db).await;

    let older_id = create_test_submission(level_a, submitter_id, db).await;
    let newer_id = create_test_submission(level_b, submitter_id, db).await;

    let t1: DateTime<Utc> = "2020-01-01T00:00:00Z".parse().unwrap();
    let t2: DateTime<Utc> = "2021-01-01T00:00:00Z".parse().unwrap();

    diesel::update(submissions::table.filter(submissions::id.eq(older_id)))
        .set((
            submissions::created_at.eq(t1),
            submissions::updated_at.eq(t1),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    diesel::update(submissions::table.filter(submissions::id.eq(newer_id)))
        .set((
            submissions::created_at.eq(t2),
            submissions::updated_at.eq(t2),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    (older_id, newer_id)
}
