use crate::app_data::db::DbAppState;
use crate::error_handler::{ApiError, StartupError};
use crate::scheduled::{sleep_until_next, startup_schedule};
use crate::schema::matview_refresh_log;
use chrono::Utc;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, RunQueryDsl};
use std::sync::Arc;
use std::time::Duration;
use tokio::task;

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = matview_refresh_log, check_for_backend(Pg))]
pub struct MatviewRefreshLog {
    pub view_name: String,
    pub last_refresh: chrono::DateTime<Utc>,
}

pub async fn start_matviews_refresher(db: Arc<DbAppState>) -> Result<(), StartupError> {
    let schedule = startup_schedule("MATVIEWS_REFRESH_SCHEDULE")?;

    let schemas = ["aredl", "arepl"];
    let views = [
        "user_leaderboard",
        "country_leaderboard",
        "clans_leaderboard",
        "country_created_levels",
        "clans_created_levels",
        "submission_stats",
        "record_totals",
        "submission_totals",
    ];

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            tracing::info!("Refreshing materialized views");

            for &schema in &schemas {
                for &view in &views {
                    let full_name = format!("{}.{}", schema, view);
                    let sql = format!("REFRESH MATERIALIZED VIEW {}", full_name);
                    match db.connection().and_then(|mut conn| {
                        diesel::sql_query(&sql)
                            .execute(&mut conn)
                            .map_err(ApiError::from)
                    }) {
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
                    if let Err(e) = db.connection().and_then(|mut conn| {
                        diesel::insert_into(matview_refresh_log::table)
                            .values(&new_timestamp)
                            .on_conflict(matview_refresh_log::view_name)
                            .do_update()
                            .set(
                                matview_refresh_log::last_refresh
                                    .eq(excluded(matview_refresh_log::last_refresh)),
                            )
                            .execute(&mut conn)
                            .map_err(ApiError::from)
                    }) {
                        tracing::error!("Couldn't log refresh for {}: {}", full_name, e);
                    }
                }
            }

            sleep_until_next(&schedule).await;
        }
    });

    Ok(())
}
