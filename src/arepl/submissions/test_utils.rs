#[cfg(test)]
use {
    crate::{
        app_data::db::DbAppState,
        arepl::{
            levels::test_utils::create_test_level,
            submissions::{history::SubmissionHistory, Submission, SubmissionStatus},
        },
        schema::arepl::{submission_history, submissions},
    },
    chrono::{DateTime, Utc},
    diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl as _, RunQueryDsl as _, SelectableHelper as _},
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
            submissions::video_url.eq("https://video.com"),
            submissions::raw_url.eq("https://raw.com"),
            submissions::priority.eq(false),
            submissions::user_notes.eq("Test submission"),
            submissions::completion_time.eq(1_000_000),
            submissions::mod_menu.eq("Mega hack"),
        ))
        .returning(submissions::id)
        .get_result::<Uuid>(&mut db.connection().unwrap())
        .expect("Failed to create test submission!")
}

#[cfg(test)]
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
        status,
        completion_time: Some(1_000_000),
        video_url: Some("https://video.com".to_owned()),
        raw_url: Some("https://raw.com".to_owned()),
        mobile: Some(false),
        ldm_id: None,
        mod_menu: Some("Mega Hack v8".to_owned()),
        timestamp: Utc::now(),
        user_notes: None,
        private_reviewer_notes: None,
        reviewer_id,
        locked: Some(false),
    };
    diesel::insert_into(submission_history::table)
        .values(&history)
        .execute(&mut db.connection().unwrap())
        .expect("Failed to insert submission history");
}

#[cfg(test)]
pub fn set_history_timestamp(db: &Arc<DbAppState>, submission_id: Uuid, timestamp: DateTime<Utc>) {
    diesel::update(
        submission_history::table.filter(submission_history::submission_id.eq(submission_id)),
    )
    .set(submission_history::timestamp.eq(timestamp))
    .execute(&mut db.connection().unwrap())
    .unwrap();
}

#[cfg(test)]
pub fn set_test_submission_reviewer(
    db: &Arc<DbAppState>,
    submission_id: Uuid,
    reviewer_id: Option<Uuid>,
) {
    diesel::update(submissions::table.filter(submissions::id.eq(submission_id)))
        .set(submissions::reviewer_id.eq(reviewer_id))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test arepl submission reviewer");
}

#[cfg(test)]
pub fn set_test_submission_reviewer_with_private_notes(
    db: &Arc<DbAppState>,
    submission_id: Uuid,
    reviewer_id: Option<Uuid>,
    private_reviewer_notes: Option<&str>,
) {
    diesel::update(submissions::table.filter(submissions::id.eq(submission_id)))
        .set((
            submissions::reviewer_id.eq(reviewer_id),
            submissions::private_reviewer_notes.eq(private_reviewer_notes.map(str::to_owned)),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test arepl submission reviewer metadata");
}

#[cfg(test)]
pub fn set_test_submission_status(
    db: &Arc<DbAppState>,
    submission_id: Uuid,
    status: SubmissionStatus,
) {
    diesel::update(submissions::table.filter(submissions::id.eq(submission_id)))
        .set(submissions::status.eq(status))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test arepl submission status");
}

#[cfg(test)]
pub fn test_submission_priorities(
    db: &Arc<DbAppState>,
    submission_ids: [Uuid; 3],
) -> std::collections::HashMap<Uuid, bool> {
    submissions::table
        .filter(submissions::id.eq_any(submission_ids))
        .select((submissions::id, submissions::priority))
        .load::<(Uuid, bool)>(&mut db.connection().unwrap())
        .expect("Failed to load test arepl submission priorities")
        .into_iter()
        .collect()
}

#[cfg(test)]
pub fn set_test_submission_raw_url_status_and_reviewer(
    db: &Arc<DbAppState>,
    submission_id: Uuid,
    raw_url: Option<&str>,
    status: SubmissionStatus,
    reviewer_id: Option<Uuid>,
) {
    diesel::update(submissions::table.filter(submissions::id.eq(submission_id)))
        .set((
            submissions::raw_url.eq(raw_url.map(str::to_owned)),
            submissions::status.eq(status),
            submissions::reviewer_id.eq(reviewer_id),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test arepl submission review state");
}

#[cfg(test)]
pub fn set_test_submission_raw_url(
    db: &Arc<DbAppState>,
    submission_id: Uuid,
    raw_url: Option<&str>,
) {
    diesel::update(submissions::table.filter(submissions::id.eq(submission_id)))
        .set(submissions::raw_url.eq(raw_url.map(str::to_owned)))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test arepl submission raw URL");
}

#[cfg(test)]
pub fn set_test_submissions_raw_url(
    db: &Arc<DbAppState>,
    submission_ids: Vec<Uuid>,
    raw_url: Option<&str>,
) {
    diesel::update(submissions::table.filter(submissions::id.eq_any(submission_ids)))
        .set(submissions::raw_url.eq(raw_url.map(str::to_owned)))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test arepl submissions raw URL");
}

#[cfg(test)]
pub fn get_test_submission(db: &Arc<DbAppState>, submission_id: Uuid) -> Submission {
    get_test_submission_optional(db, submission_id).expect("Failed to fetch test arepl submission")
}

#[cfg(test)]
pub fn get_test_submission_optional(
    db: &Arc<DbAppState>,
    submission_id: Uuid,
) -> Option<Submission> {
    submissions::table
        .find(submission_id)
        .select(Submission::as_select())
        .first(&mut db.connection().unwrap())
        .optional()
        .expect("Failed to fetch test arepl submission")
}

#[cfg(test)]
pub fn latest_test_submission_history(
    db: &Arc<DbAppState>,
    submission_id: Uuid,
) -> SubmissionHistory {
    submission_history::table
        .filter(submission_history::submission_id.eq(submission_id))
        .order(submission_history::timestamp.desc())
        .select(SubmissionHistory::as_select())
        .first::<SubmissionHistory>(&mut db.connection().unwrap())
        .expect("Failed to get test arepl submission history")
}

#[cfg(test)]
pub fn set_test_submission_history_reviewer(
    db: &Arc<DbAppState>,
    submission_id: Uuid,
    reviewer_id: Option<Uuid>,
) {
    diesel::update(
        submission_history::table.filter(submission_history::submission_id.eq(submission_id)),
    )
    .set(submission_history::reviewer_id.eq(reviewer_id))
    .execute(&mut db.connection().unwrap())
    .expect("Failed to set test arepl submission history reviewer");
}

#[cfg(test)]
pub async fn create_two_test_submissions_with_different_timestamps(
    db: &Arc<DbAppState>,
    submitter_id: uuid::Uuid,
) -> (uuid::Uuid, uuid::Uuid) {
    let level_a = create_test_level(db).await;
    let level_b = create_test_level(db).await;

    let sub_a = create_test_submission(level_a, submitter_id, db).await;
    let sub_b = create_test_submission(level_b, submitter_id, db).await;

    let t1: DateTime<Utc> = "2020-01-01T00:00:00Z".parse().unwrap();
    let t2: DateTime<Utc> = "2021-01-01T00:00:00Z".parse().unwrap();

    let conn = &mut db.connection().unwrap();

    diesel::update(submissions::table.filter(submissions::id.eq(sub_a)))
        .set((
            submissions::created_at.eq(t1),
            submissions::updated_at.eq(t1),
            submissions::completion_time.eq(10_000_i64),
        ))
        .execute(conn)
        .unwrap();

    diesel::update(submissions::table.filter(submissions::id.eq(sub_b)))
        .set((
            submissions::created_at.eq(t2),
            submissions::updated_at.eq(t2),
            submissions::completion_time.eq(5_000_i64),
        ))
        .execute(conn)
        .unwrap();

    (sub_a, sub_b)
}
