use crate::db::DbAppState;
use crate::get_secret;
use crate::schema::matview_refresh_log;
use chrono::Utc;
use cron::Schedule;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, RunQueryDsl};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;

use crate::aredl::leaderboard::MatviewRefreshLog;

pub fn start_leaderboard_refresher(db: Arc<DbAppState>) {
    let schedule = Schedule::from_str(&get_secret("LEADERBOARD_REFRESH_SCHEDULE")).unwrap();
    let schedule = Arc::new(schedule);
    let db_clone = db.clone();

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            tracing::info!("Refreshing leaderboard");

            let conn_result = db_clone.connection();

            if conn_result.is_err() {
                tracing::error!("Failed to refresh {}", conn_result.err().unwrap());
                continue;
            }

            let mut conn = conn_result.unwrap();

            let result = diesel::sql_query("REFRESH MATERIALIZED VIEW aredl_user_leaderboard")
                .execute(&mut conn);

            let now = Utc::now();
            let new_stamp = MatviewRefreshLog {
                view_name: "aredl_user_leaderboard".to_string(),
                last_refresh: now,
            };

            diesel::insert_into(matview_refresh_log::table)
                .values(&new_stamp)
                .on_conflict(matview_refresh_log::view_name)
                .do_update()
                .set(
                    matview_refresh_log::last_refresh
                        .eq(excluded(matview_refresh_log::last_refresh)),
                )
                .execute(&mut conn)
                .map_err(|e| tracing::error!("Failed to upsert refresh log: {}", e))
                .ok();

            if result.is_err() {
                tracing::error!(
                    "Failed to refresh user leaderboard {}",
                    result.err().unwrap()
                )
            }

            let result = diesel::sql_query("REFRESH MATERIALIZED VIEW aredl_country_leaderboard")
                .execute(&mut conn);

            let now = Utc::now();
            let new_stamp = MatviewRefreshLog {
                view_name: "aredl_country_leaderboard".to_string(),
                last_refresh: now,
            };

            diesel::insert_into(matview_refresh_log::table)
                .values(&new_stamp)
                .on_conflict(matview_refresh_log::view_name)
                .do_update()
                .set(
                    matview_refresh_log::last_refresh
                        .eq(excluded(matview_refresh_log::last_refresh)),
                )
                .execute(&mut conn)
                .map_err(|e| tracing::error!("Failed to upsert refresh log: {}", e))
                .ok();

            if result.is_err() {
                tracing::error!(
                    "Failed to refresh country leaderboard {}",
                    result.err().unwrap()
                )
            }

            let result = diesel::sql_query("REFRESH MATERIALIZED VIEW aredl_clans_leaderboard")
                .execute(&mut conn);

            let now = Utc::now();
            let new_stamp = MatviewRefreshLog {
                view_name: "aredl_clans_leaderboard".to_string(),
                last_refresh: now,
            };

            diesel::insert_into(matview_refresh_log::table)
                .values(&new_stamp)
                .on_conflict(matview_refresh_log::view_name)
                .do_update()
                .set(
                    matview_refresh_log::last_refresh
                        .eq(excluded(matview_refresh_log::last_refresh)),
                )
                .execute(&mut conn)
                .map_err(|e| tracing::error!("Failed to upsert refresh log: {}", e))
                .ok();

            if result.is_err() {
                tracing::error!(
                    "Failed to refresh clans leaderboard {}",
                    result.err().unwrap()
                )
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;
        }
    });
}
