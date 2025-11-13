use crate::app_data::db::DbConnection;
use crate::schema::arepl::position_history;
use chrono::Utc;
use diesel::{ExpressionMethods, RunQueryDsl};
use uuid::Uuid;

#[cfg(test)]
pub fn insert_history_entry(
    conn: &mut DbConnection,
    new_position: Option<i32>,
    old_position: Option<i32>,
    legacy: Option<bool>,
    affected_level: Uuid,
    level_above: Option<Uuid>,
    level_below: Option<Uuid>,
) {
    diesel::insert_into(position_history::table)
        .values((
            position_history::new_position.eq(new_position),
            position_history::old_position.eq(old_position),
            position_history::legacy.eq(legacy),
            position_history::affected_level.eq(affected_level),
            position_history::level_above.eq(level_above),
            position_history::level_below.eq(level_below),
            position_history::created_at.eq(Utc::now()),
        ))
        .execute(conn)
        .expect("Failed to insert history entry");
}
