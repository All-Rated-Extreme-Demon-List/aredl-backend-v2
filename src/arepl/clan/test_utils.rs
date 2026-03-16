#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use crate::app_data::db::DbAppState;
#[cfg(test)]
use diesel::RunQueryDsl;

#[cfg(test)]
pub async fn refresh_test_clan_created_levels(db: &Arc<DbAppState>) {
    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.clans_created_levels")
        .execute(&mut db.connection().unwrap())
        .expect("Failed to update clans created levels");
}
