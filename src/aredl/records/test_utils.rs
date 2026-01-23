#[cfg(test)]
use {
    crate::{
        app_data::db::DbAppState,
        aredl::{levels::test_utils::create_test_level_with_record, submissions::SubmissionStatus},
        schema::aredl::{records, submissions},
    },
    chrono::{DateTime, Utc},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    std::sync::Arc,
    uuid::Uuid,
};
#[cfg(test)]
pub async fn create_test_record(db: &Arc<DbAppState>, user_id: Uuid, level_id: Uuid) -> Uuid {
    let conn = &mut db.connection().unwrap();
    let submission_id = diesel::insert_into(submissions::table)
        .values((
            submissions::submitted_by.eq(user_id),
            submissions::video_url.eq("https://youtube.com/watch?v=xvFZjo5PgG0"),
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

pub async fn create_two_test_records_with_different_timestamps(
    db: &Arc<DbAppState>,
    user_id: uuid::Uuid,
) -> (uuid::Uuid, uuid::Uuid) {
    let (_level_a, record_a) = create_test_level_with_record(db, user_id).await;
    let (_level_b, record_b) = create_test_level_with_record(db, user_id).await;

    let t1: DateTime<Utc> = "2020-01-01T00:00:00Z".parse().unwrap();
    let t2: DateTime<Utc> = "2021-01-01T00:00:00Z".parse().unwrap();

    // record_a older, record_b newer
    diesel::update(records::table.filter(records::id.eq(record_a)))
        .set((
            records::created_at.eq(t1),
            records::updated_at.eq(t1),
            records::achieved_at.eq(t1),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    diesel::update(records::table.filter(records::id.eq(record_b)))
        .set((
            records::created_at.eq(t2),
            records::updated_at.eq(t2),
            records::achieved_at.eq(t2),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    (record_a, record_b)
}
