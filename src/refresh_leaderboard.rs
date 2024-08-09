use std::str::FromStr;
use std::sync::Arc;
use cron::Schedule;
use diesel::RunQueryDsl;
use crate::db::{DbAppState};
use crate::error_handler::ApiError;

pub fn start_leaderboard_refresher() {
    let schedule = Schedule::from_str("0 * * * * *").unwrap();
    let schedule = Arc::new(schedule);

}

pub fn refresh_leaderboard_fn(db: Arc<DbAppState>) -> impl Fn() -> Result<(), ApiError> {
    return move || {
        diesel::sql_query("ALTER TABLE aredl_levels ENABLE TRIGGER aredl_level_place")
            .execute(&mut db.connection()?)?;
        Ok(())
    }
}