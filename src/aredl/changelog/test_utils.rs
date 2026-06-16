#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use crate::app_data::db::DbAppState;
use crate::aredl::levels::LevelStatus;
use crate::schema::aredl::position_history;
use chrono::Utc;
use diesel::{ExpressionMethods, RunQueryDsl};
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub fn insert_history_entry(
    db: &Arc<DbAppState>,
    new_position: Option<i32>,
    old_position: Option<i32>,
    old_status: Option<LevelStatus>,
    new_status: LevelStatus,
    affected_level: Uuid,
    level_above: Option<Uuid>,
    level_below: Option<Uuid>,
) {
    diesel::insert_into(position_history::table)
        .values((
            position_history::new_position.eq(new_position),
            position_history::old_position.eq(old_position),
            position_history::old_status.eq(old_status),
            position_history::new_status.eq(new_status),
            position_history::affected_level.eq(affected_level),
            position_history::level_above.eq(level_above),
            position_history::level_below.eq(level_below),
            position_history::created_at.eq(Utc::now()),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to insert history entry");
}
