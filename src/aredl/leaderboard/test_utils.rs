#[cfg(test)]
use crate::app_data::db::DbConnection;
#[cfg(test)]
use diesel::RunQueryDsl;

#[cfg(test)]
pub async fn refresh_test_leaderboards(conn: &mut DbConnection) {
    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.user_leaderboard")
        .execute(conn)
        .expect("Failed to update leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.country_leaderboard")
        .execute(conn)
        .expect("Failed to update country leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.clans_leaderboard")
        .execute(conn)
        .expect("Failed to update clans leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.position_history_full_view")
        .execute(conn)
        .expect("Failed to update position history");
}
