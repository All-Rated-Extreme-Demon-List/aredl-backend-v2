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

pub async fn start_leaderboard_refresher(db: Arc<DbAppState>) {
    let schedule = Schedule::from_str(&get_secret("LEADERBOARD_REFRESH_SCHEDULE")).unwrap();
    let schedule = Arc::new(schedule);
    let db_clone = db.clone();

    let schemas = ["aredl", "arepl"];
    let views = [
        "user_leaderboard",
        "country_leaderboard",
        "clans_leaderboard",
    ];

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

            for &schema in &schemas {
                for &view in &views {
                    let full_name = format!("{}.{}", schema, view);
                    let sql = format!("REFRESH MATERIALIZED VIEW {}", full_name);
                    match diesel::sql_query(&sql).execute(&mut conn) {
                        Ok(_) => {
                            tracing::info!("Refreshed {}", full_name)
                        }
                        Err(e) => {
                            tracing::error!("Failed to refresh {}: {}", full_name, e)
                        }
                    }

                    let new_timestamp = MatviewRefreshLog {
                        view_name: full_name.clone(),
                        last_refresh: Utc::now(),
                    };
                    if let Err(e) = diesel::insert_into(matview_refresh_log::table)
                        .values(&new_timestamp)
                        .on_conflict(matview_refresh_log::view_name)
                        .do_update()
                        .set(
                            matview_refresh_log::last_refresh
                                .eq(excluded(matview_refresh_log::last_refresh)),
                        )
                        .execute(&mut conn)
                    {
                        tracing::error!("Couldn't log refresh for {}: {}", full_name, e);
                    }
                }
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;
        }
    });
}
