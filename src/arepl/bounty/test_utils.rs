#[cfg(test)]
use {
    crate::{
        app_data::db::DbAppState,
        arepl::{
            bounty::{Bounty, BountyDifficulty, BountyPost, BountyType},
            records::Record,
        },
        schema::arepl::{bounties, bounty_completed, records},
    },
    chrono::{DateTime, Utc},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper},
    serde_json::Value,
    std::sync::Arc,
    uuid::Uuid,
};

#[cfg(test)]
pub async fn create_test_bounty(
    db: &Arc<DbAppState>,
    level_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: Option<DateTime<Utc>>,
    target_submissions: Option<i32>,
    is_target_public: bool,
) -> Bounty {
    create_test_bounty_with_type(
        db,
        level_id,
        BountyType::Bounty,
        start_date,
        end_date,
        target_submissions,
        is_target_public,
    )
    .await
}

#[cfg(test)]
pub async fn create_test_bounty_with_type(
    db: &Arc<DbAppState>,
    level_id: Uuid,
    bounty_type: BountyType,
    start_date: DateTime<Utc>,
    end_date: Option<DateTime<Utc>>,
    target_submissions: Option<i32>,
    is_target_public: bool,
) -> Bounty {
    Bounty::create(
        &mut db.connection().unwrap(),
        BountyPost {
            level_id,
            bounty_type,
            bounty_difficulty: BountyDifficulty::Medium,
            start_date,
            end_date,
            target_submissions,
            is_target_public,
        },
    )
    .expect("Failed to create test bounty")
}

#[cfg(test)]
pub async fn create_test_bounty_completion(db: &Arc<DbAppState>, bounty_id: Uuid, user_id: Uuid) {
    diesel::insert_into(bounty_completed::table)
        .values((
            bounty_completed::bounty_id.eq(bounty_id),
            bounty_completed::user_id.eq(user_id),
            bounty_completed::completed_at.eq(Utc::now()),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test bounty completion");
}

#[cfg(test)]
pub fn count_test_bounty_completions(db: &Arc<DbAppState>, bounty_id: Uuid) -> i64 {
    bounty_completed::table
        .filter(bounty_completed::bounty_id.eq(bounty_id))
        .count()
        .get_result(&mut db.connection().unwrap())
        .expect("Failed to count test bounty completions")
}

#[cfg(test)]
pub fn fetch_test_bounty(db: &Arc<DbAppState>, bounty_id: Uuid) -> Bounty {
    bounties::table
        .filter(bounties::id.eq(bounty_id))
        .first::<Bounty>(&mut db.connection().unwrap())
        .expect("Failed to fetch test bounty")
}

#[cfg(test)]
pub fn fetch_test_record(db: &Arc<DbAppState>, record_id: Uuid) -> Record {
    records::table
        .filter(records::id.eq(record_id))
        .select(Record::as_select())
        .first::<Record>(&mut db.connection().unwrap())
        .expect("Failed to fetch test record")
}

#[cfg(test)]
pub fn set_test_record_achieved_at(
    db: &Arc<DbAppState>,
    record_id: Uuid,
    achieved_at: DateTime<Utc>,
) {
    diesel::update(records::table.filter(records::id.eq(record_id)))
        .set(records::achieved_at.eq(achieved_at))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to update test record achieved_at");
}

#[cfg(test)]
pub fn find_test_bounty(body: &Value, bounty_id: Uuid) -> &Value {
    body.as_array()
        .expect("bounty board response must be an array")
        .iter()
        .find(|bounty| bounty["id"] == bounty_id.to_string())
        .expect("bounty must be present in response")
}
