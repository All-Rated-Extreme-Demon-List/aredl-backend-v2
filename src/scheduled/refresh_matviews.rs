use crate::{
    app_data::db::DbAppState,
    error_handler::{ApiError, StartupError},
    scheduled::{sleep_until_next, startup_schedule},
    schema::matview_refresh_log,
};
use actix_web::web;
use chrono::Utc;
use diesel::{prelude::*, upsert::excluded};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use strum::IntoEnumIterator as _;
use strum_macros::{Display, EnumIter};
use tokio::task;
use utoipa::ToSchema;

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = matview_refresh_log, check_for_backend(Pg))]
pub struct MatviewRefreshLog {
    pub view_name: String,
    pub last_refresh: chrono::DateTime<Utc>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Display, EnumIter, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Matview {
    UserLeaderboard,
    CountryLeaderboard,
    ClansLeaderboard,
    CountryCreatedLevels,
    ClansCreatedLevels,
    RecordTotals,
    SubmissionTotals,
}

pub async fn start_matviews_refresher(db: Arc<DbAppState>) -> Result<(), StartupError> {
    let schedule = startup_schedule("MATVIEWS_REFRESH_SCHEDULE")?;
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            for view in Matview::iter() {
                if let Err(error) = refresh(db.clone(), view).await {
                    tracing::error!("Failed to refresh materialized view {view}: {error}");
                }
            }
            sleep_until_next(&schedule).await;
        }
    });
    Ok(())
}

pub async fn refresh(db: Arc<DbAppState>, view: Matview) -> Result<(), ApiError> {
    web::block(move || {
        let conn = &mut db.connection()?;
        conn.transaction::<_, ApiError, _>(|conn| {
            diesel::sql_query("SET LOCAL work_mem = '64MB'").execute(conn)?;
            diesel::sql_query(format!("REFRESH MATERIALIZED VIEW CONCURRENTLY {view}"))
                .execute(conn)?;
            let new_timestamp = MatviewRefreshLog {
                view_name: view.to_string(),
                last_refresh: Utc::now(),
            };
            diesel::insert_into(matview_refresh_log::table)
                .values(&new_timestamp)
                .on_conflict(matview_refresh_log::view_name)
                .do_update()
                .set(
                    matview_refresh_log::last_refresh
                        .eq(excluded(matview_refresh_log::last_refresh)),
                )
                .execute(conn)?;
            Ok(())
        })
    })
    .await?
}
