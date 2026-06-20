use crate::app_data::db::DbAppState;
use crate::error_handler::StartupError;
use crate::notifications::WebsocketNotification;
use crate::scheduled::{sleep_until_next, startup_schedule};
use crate::schema::shifts;
use crate::shifts::Shift;
use crate::shifts::ShiftStatus;

use chrono::Utc;
use diesel::query_dsl::methods::FilterDsl as _;
use diesel::{ExpressionMethods as _, RunQueryDsl as _};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task;

pub async fn start_data_cleaner(
    db: Arc<DbAppState>,
    notify_tx: broadcast::Sender<WebsocketNotification>,
) -> Result<(), StartupError> {
    let schedule = startup_schedule("DATA_CLEANER_SCHEDULE")?;

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            tracing::info!("Running data cleaner");
            {
                let conn = &mut match db.connection() {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("DB connection failed: {e}");
                        continue;
                    }
                };

                tracing::info!("Cleaning old notifications");

                if let Err(error) = diesel::sql_query(
                    "DELETE FROM notifications WHERE created_at < NOW() - INTERVAL '1 month'",
                )
                .execute(conn)
                {
                    tracing::error!("Failed to clean notifications {error}");
                }

                tracing::info!("Cleaning stale submissions claims");

                if let Err(error) = diesel::sql_query(
                    "UPDATE aredl.submissions \
                 SET status = 'Pending', reviewer_id = NULL \
                 WHERE status = 'Claimed' \
                   AND updated_at < NOW() - INTERVAL '120 minutes';",
                )
                .execute(conn)
                {
                    tracing::error!("Failed to clean stale submissions claims for AREDL: {error}");
                }

                if let Err(error) = diesel::sql_query(
                    "UPDATE arepl.submissions \
                 SET status = 'Pending', reviewer_id = NULL \
                 WHERE status = 'Claimed' \
                   AND updated_at < NOW() - INTERVAL '120 minutes';",
                )
                .execute(conn)
                {
                    tracing::error!("Failed to clean stale submissions claims for AREPL: {error}");
                }

                tracing::info!("Expiring overdue shifts");

                let aredl_expired_shifts: Vec<Shift> = shifts::table
                    .filter(shifts::status.eq(ShiftStatus::Running))
                    .filter(shifts::end_at.lt(Utc::now()))
                    .load(conn)
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to load expired shifts: {}", e);
                        vec![]
                    });

                if let Err(e) = diesel::update(
                    shifts::table
                        .filter(shifts::status.eq(ShiftStatus::Running))
                        .filter(shifts::end_at.lt(Utc::now())),
                )
                .set((
                    shifts::status.eq(ShiftStatus::Expired),
                    shifts::updated_at.eq(Utc::now()),
                ))
                .execute(conn)
                {
                    tracing::error!("Failed to expire shifts: {}", e);
                }

                let missed_shifts_payload = serde_json::json!({
                    "aredl": aredl_expired_shifts,
                });

                WebsocketNotification::send(&notify_tx, "SHIFTS_MISSED", &missed_shifts_payload);

                tracing::info!("Cleaned data successfully");
            }

            sleep_until_next(&schedule).await;
        }
    });

    Ok(())
}
