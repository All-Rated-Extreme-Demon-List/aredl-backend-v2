#[cfg(test)]
use {crate::app_data::db::DbAppState, diesel::prelude::*, diesel::sql_query, std::sync::Arc};

#[cfg(test)]
pub async fn refresh_test_submission_stats(db: &Arc<DbAppState>) {
    sql_query("SELECT aredl.rebuild_submission_daily_stats()")
        .execute(&mut db.connection().unwrap())
        .expect("Failed to refresh submission stats");
}
