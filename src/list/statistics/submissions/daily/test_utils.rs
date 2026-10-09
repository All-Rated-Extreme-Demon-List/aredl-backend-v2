#[cfg(test)]
use {
    crate::{app_data::db::DbAppState, list::List},
    diesel::{prelude::*, sql_query},
    std::sync::Arc,
};

#[cfg(test)]
pub async fn refresh_test_submission_stats(db: &Arc<DbAppState>) {
    sql_query("SELECT rebuild_submission_daily_stats($1)")
        .bind::<diesel::sql_types::SmallInt, _>(List::Classic)
        .execute(&mut db.connection().unwrap())
        .expect("Failed to refresh submission stats");
}
