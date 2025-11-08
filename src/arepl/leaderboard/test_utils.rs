#[cfg(test)]
use crate::db::DbConnection;
#[cfg(test)]
use diesel::RunQueryDsl;

#[cfg(test)]
pub async fn refresh_test_leaderboards(conn: &mut DbConnection) {
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
