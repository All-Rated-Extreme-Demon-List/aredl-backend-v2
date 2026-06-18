#[cfg(test)]
use {crate::app_data::db::DbAppState, diesel::RunQueryDsl, std::sync::Arc};

#[cfg(test)]
pub fn refresh_test_record_totals(db: &Arc<DbAppState>) {
    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.record_totals")
        .execute(&mut db.connection().unwrap())
        .expect("Failed to refresh arepl record totals");
}
