#[cfg(test)]
use {
    crate::{
        app_data::db::DbAppState,
        list::{
            levels::test_utils::create_test_level_with_record_for_list, records::Record,
            submissions::SubmissionStatus, List,
        },
        schema::{records, submissions},
    },
    chrono::{DateTime, Utc},
    diesel::prelude::*,
    std::sync::Arc,
    uuid::Uuid,
};

#[cfg(test)]
pub async fn create_test_record(db: &Arc<DbAppState>, user_id: Uuid, level_id: Uuid) -> Uuid {
    create_test_record_for_list(db, List::Classic, user_id, level_id).await
}

#[cfg(test)]
pub async fn create_test_record_for_list(
    db: &Arc<DbAppState>,
    list: List,
    user_id: Uuid,
    level_id: Uuid,
) -> Uuid {
    let conn = &mut db.connection().unwrap();
    let submission_id = diesel::insert_into(submissions::table)
        .values((
            submissions::list_id.eq(list),
            submissions::completion_time.eq((list == List::Platformer).then_some(1000_i64)),
            submissions::submitted_by.eq(user_id),
            submissions::video_url.eq("https://youtube.com/watch?v=xvFZjo5PgG0"),
            submissions::level_id.eq(level_id),
            submissions::status.eq(SubmissionStatus::Accepted),
            submissions::mobile.eq(false),
        ))
        .returning(submissions::id)
        .get_result::<Uuid>(conn)
        .expect("Failed to create test record");

    records::table
        .filter(records::submission_id.eq(submission_id))
        .select(records::id)
        .first::<Uuid>(conn)
        .expect("Failed to retrieve test record ID")
}

#[cfg(test)]
pub fn get_test_record(db: &Arc<DbAppState>, record_id: Uuid) -> Record {
    records::table
        .filter(records::id.eq(record_id))
        .select(Record::as_select())
        .first::<Record>(&mut db.connection().unwrap())
        .expect("Failed to get test record")
}

#[cfg(test)]
pub fn get_test_record_for_level_and_user(
    db: &Arc<DbAppState>,
    level_id: Uuid,
    user_id: Uuid,
) -> Record {
    records::table
        .filter(records::level_id.eq(level_id))
        .filter(records::submitted_by.eq(user_id))
        .select(Record::as_select())
        .first::<Record>(&mut db.connection().unwrap())
        .expect("Failed to get test record for level and user")
}

#[cfg(test)]
pub fn count_test_records_with_achieved_at(
    db: &Arc<DbAppState>,
    achieved_at: DateTime<Utc>,
) -> i64 {
    records::table
        .filter(records::achieved_at.eq(achieved_at))
        .count()
        .get_result(&mut db.connection().unwrap())
        .expect("Failed to count test records with achieved_at")
}

#[cfg(test)]
pub async fn create_two_test_records_with_different_timestamps(
    db: &Arc<DbAppState>,
    user_id: uuid::Uuid,
) -> (uuid::Uuid, uuid::Uuid) {
    create_two_test_records_with_different_timestamps_for_list(db, List::Classic, user_id).await
}

#[cfg(test)]
pub async fn create_two_test_records_with_different_timestamps_for_list(
    db: &Arc<DbAppState>,
    list: List,
    user_id: Uuid,
) -> (Uuid, Uuid) {
    let (_level_a, record_a) = create_test_level_with_record_for_list(db, list, user_id).await;
    let (_level_b, record_b) = create_test_level_with_record_for_list(db, list, user_id).await;

    let t1: DateTime<Utc> = "2020-01-01T00:00:00Z".parse().unwrap();
    let t2: DateTime<Utc> = "2021-01-01T00:00:00Z".parse().unwrap();

    // record_a older, record_b newer
    diesel::update(records::table.filter(records::id.eq(record_a)))
        .set((
            records::created_at.eq(t1),
            records::updated_at.eq(t1),
            records::achieved_at.eq(t1),
            records::completion_time.eq((list == List::Platformer).then_some(10_000_i64)),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    diesel::update(records::table.filter(records::id.eq(record_b)))
        .set((
            records::created_at.eq(t2),
            records::updated_at.eq(t2),
            records::achieved_at.eq(t2),
            records::completion_time.eq((list == List::Platformer).then_some(5_000_i64)),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    (record_a, record_b)
}

#[cfg(test)]
pub async fn set_test_record_verification(
    db: &Arc<DbAppState>,
    record_id: Uuid,
    is_verification: bool,
) {
    diesel::update(records::table.filter(records::id.eq(record_id)))
        .set(records::is_verification.eq(is_verification))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to update test arepl record verification status");
}

#[cfg(test)]
pub async fn set_test_record_mobile(db: &Arc<DbAppState>, record_id: Uuid, mobile: bool) {
    diesel::update(records::table.filter(records::id.eq(record_id)))
        .set(records::mobile.eq(mobile))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to update test arepl record mobile flag");
}

#[cfg(test)]
pub async fn set_test_record_achieved_at(
    db: &Arc<DbAppState>,
    record_id: Uuid,
    achieved_at: DateTime<Utc>,
) {
    diesel::update(records::table.filter(records::id.eq(record_id)))
        .set(records::achieved_at.eq(achieved_at))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to update test arepl record achieved_at");
}

#[cfg(test)]
pub fn test_records_for_user(db: &Arc<DbAppState>, user_id: Uuid) -> Vec<Record> {
    records::table
        .filter(records::submitted_by.eq(user_id))
        .select(Record::as_select())
        .get_results::<Record>(&mut db.connection().unwrap())
        .expect("Failed to collect test aredl records for user")
}

#[cfg(test)]
pub fn test_records_for_level(db: &Arc<DbAppState>, level_id: Uuid) -> Vec<Record> {
    records::table
        .filter(records::level_id.eq(level_id))
        .select(Record::as_select())
        .get_results::<Record>(&mut db.connection().unwrap())
        .expect("Failed to collect test aredl records for level")
}
