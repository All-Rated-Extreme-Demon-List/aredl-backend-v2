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

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = matview_refresh_log, check_for_backend(Pg))]
pub struct MatviewRefreshLog {
    pub view_name: String,
    pub last_refresh: chrono::DateTime<Utc>,
}

pub async fn start_matviews_refresher(db: Arc<DbAppState>) {
    let schedule = Schedule::from_str(&get_secret("MATVIEWS_REFRESH_SCHEDULE")).unwrap();
    let schedule = Arc::new(schedule);

    let schemas = ["aredl", "arepl"];
    let views = [
        "user_leaderboard",
        "country_leaderboard",
        "clans_leaderboard",
        "submission_stats",
    ];

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            tracing::info!("Refreshing materialized views");

            let conn = &mut match db.connection() {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("DB connection failed: {e}");
                    continue;
                }
            };

            for &schema in &schemas {
                for &view in &views {
                    let full_name = format!("{}.{}", schema, view);
                    let sql = format!("REFRESH MATERIALIZED VIEW {}", full_name);
                    match diesel::sql_query(&sql).execute(conn) {
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
                        .execute(conn)
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
