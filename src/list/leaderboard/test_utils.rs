#[cfg(test)]
use {crate::app_data::db::DbAppState, diesel::prelude::*, std::sync::Arc};

#[cfg(test)]
pub async fn refresh_test_leaderboards(db: &Arc<DbAppState>) {
    let conn = &mut db.connection().unwrap();
    diesel::sql_query("REFRESH MATERIALIZED VIEW public.user_leaderboard")
        .execute(conn)
        .expect("Failed to update leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW public.country_leaderboard")
        .execute(conn)
        .expect("Failed to update country leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW public.clans_leaderboard")
        .execute(conn)
        .expect("Failed to update clans leaderboard");
}
