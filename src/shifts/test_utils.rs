#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use crate::{
    app_data::db::DbAppState,
    schema::{recurrent_shifts, shifts},
    shifts::{Shift, Weekday},
};
#[cfg(test)]
use chrono::Utc;
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_shift(
    db: &Arc<DbAppState>,
    user_id: Uuid,
    should_start_immediately: bool,
) -> Uuid {
    let start_time = match should_start_immediately {
        true => Utc::now(),
        false => Utc::now() + chrono::Duration::hours(1),
    };

    diesel::insert_into(shifts::table)
        .values((
            shifts::user_id.eq(user_id),
            shifts::target_count.eq(20),
            shifts::start_at.eq(start_time),
            shifts::end_at.eq(start_time + chrono::Duration::hours(4)),
        ))
        .returning(shifts::id)
        .get_result::<Uuid>(&mut db.connection().unwrap())
        .expect("Failed to create test shift")
}
#[cfg(test)]
pub async fn create_test_recurring_shift(db: &Arc<DbAppState>, user_id: Uuid) -> Uuid {
    diesel::insert_into(recurrent_shifts::table)
        .values((
            recurrent_shifts::user_id.eq(user_id),
            recurrent_shifts::start_hour.eq(12),
            recurrent_shifts::target_count.eq(20),
            recurrent_shifts::duration.eq(1),
            recurrent_shifts::weekday.eq(Weekday::Friday),
        ))
        .returning(recurrent_shifts::id)
        .get_result::<Uuid>(&mut db.connection().unwrap())
        .expect("Failed to create test shift")
}

#[cfg(test)]
pub async fn set_test_shift_target_count(db: &Arc<DbAppState>, shift_id: Uuid, target_count: i32) {
    diesel::update(shifts::table.filter(shifts::id.eq(shift_id)))
        .set(shifts::target_count.eq(target_count))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test shift target count");
}

#[cfg(test)]
pub fn get_test_shift(db: &Arc<DbAppState>, shift_id: Uuid) -> Shift {
    shifts::table
        .find(shift_id)
        .first(&mut db.connection().unwrap())
        .expect("Failed to get test shift")
}

#[cfg(test)]
pub fn test_shifts_for_user(db: &Arc<DbAppState>, user_id: Uuid) -> Vec<Shift> {
    shifts::table
        .filter(shifts::user_id.eq(user_id))
        .load(&mut db.connection().unwrap())
        .expect("Failed to load test shifts for user")
}
