#[cfg(test)]
use {
    crate::{app_data::db::DbAppState, schema::matview_refresh_log},
    chrono::{DateTime, Utc},
    diesel::prelude::*,
    std::sync::Arc,
};

pub fn get_matview_refresh_time(db: &Arc<DbAppState>, view: &str) -> DateTime<Utc> {
    matview_refresh_log::table
        .find(view)
        .select(matview_refresh_log::last_refresh)
        .first(&mut db.connection().unwrap())
        .expect("Missing materialized view refresh timestamp")
}
