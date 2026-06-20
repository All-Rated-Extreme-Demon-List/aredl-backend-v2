#[cfg(test)]
use {crate::app_data::db::DbAppState, diesel::RunQueryDsl as _, std::sync::Arc};

#[cfg(test)]
pub fn refresh_test_submission_totals(db: &Arc<DbAppState>) {
    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.submission_totals")
        .execute(&mut db.connection().unwrap())
        .expect("Failed to refresh aredl submission totals");
}
