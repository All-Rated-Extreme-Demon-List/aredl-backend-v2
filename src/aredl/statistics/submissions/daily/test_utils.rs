#[cfg(test)]
use {
    crate::app_data::db::DbAppState,
    diesel::{sql_query, RunQueryDsl as _},
    std::sync::Arc,
};

#[cfg(test)]
pub async fn refresh_test_submission_stats(db: &Arc<DbAppState>) {
    sql_query("REFRESH MATERIALIZED VIEW aredl.submission_stats")
        .execute(&mut db.connection().unwrap())
        .expect("Failed to refresh submission stats");
}
