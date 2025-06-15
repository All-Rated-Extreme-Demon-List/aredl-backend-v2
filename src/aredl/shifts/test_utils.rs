#[cfg(test)]
use crate::{
    aredl::shifts::Weekday,
    db::DbConnection,
    schema::aredl::{recurrent_shifts, shifts},
};
#[cfg(test)]
use chrono::Utc;
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_shift(
    conn: &mut DbConnection,
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
        .get_result::<Uuid>(conn)
        .expect("Failed to create test shift")
}
#[cfg(test)]
pub async fn create_test_recurring_shift(conn: &mut DbConnection, user_id: Uuid) -> Uuid {
    diesel::insert_into(recurrent_shifts::table)
        .values((
            recurrent_shifts::user_id.eq(user_id),
            recurrent_shifts::start_hour.eq(12),
            recurrent_shifts::target_count.eq(20),
            recurrent_shifts::duration.eq(1),
            recurrent_shifts::weekday.eq(Weekday::Friday),
        ))
        .returning(recurrent_shifts::id)
        .get_result::<Uuid>(conn)
        .expect("Failed to create test shift")
}
