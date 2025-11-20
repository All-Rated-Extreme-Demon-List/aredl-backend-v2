#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use crate::app_data::db::DbAppState;
#[cfg(test)]
use diesel::RunQueryDsl;

#[cfg(test)]
pub async fn refresh_test_leaderboards(db: &Arc<DbAppState>) {
    let conn = &mut db.connection().unwrap();
    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.user_leaderboard")
        .execute(conn)
        .expect("Failed to update leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.country_leaderboard")
        .execute(conn)
        .expect("Failed to update country leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.clans_leaderboard")
        .execute(conn)
        .expect("Failed to update clans leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.position_history_full_view")
        .execute(conn)
        .expect("Failed to update position history");
}
