use crate::app_data::db::DbAppState;
use crate::error_handler::StartupError;
use crate::list::submissions::SubmissionStatus;
use crate::notifications::WebsocketNotification;
use crate::scheduled::{sleep_until_next, startup_schedule};
use crate::schema::{notifications, shifts, submissions};
use crate::shifts::Shift;
use crate::shifts::ShiftStatus;

use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task;
use uuid::Uuid;

use diesel::prelude::*;
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

                diesel::delete(
                    notifications::table.filter(
                        notifications::created_at.lt(Utc::now() - chrono::Duration::days(30)),
                    ),
                )
                .execute(conn)
                .unwrap_or_else(|error| {
                    tracing::error!("Failed to clean stale notifications: {error}",);
                    0
                });

                tracing::info!("Cleaning stale submissions claims");

                diesel::update(
                    submissions::table
                        .filter(submissions::status.eq(SubmissionStatus::Claimed))
                        .filter(
                            submissions::updated_at.lt(Utc::now() - chrono::Duration::minutes(120)),
                        ),
                )
                .set((
                    submissions::status.eq(SubmissionStatus::Pending),
                    submissions::reviewer_id.eq(None::<Uuid>),
                ))
                .execute(conn)
                .unwrap_or_else(|error| {
                    tracing::error!("Failed to clean stale submissions claims: {error}",);
                    0
                });

                tracing::info!("Expiring overdue shifts");

                let expired_shifts: Vec<Shift> = shifts::table
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

                let missed_shifts_payload = serde_json::json!(expired_shifts);

                WebsocketNotification::send(&notify_tx, "SHIFTS_MISSED", &missed_shifts_payload);

                tracing::info!("Cleaned data successfully");
            }

            sleep_until_next(&schedule).await;
        }
    });

    Ok(())
}
