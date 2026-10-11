use crate::{
    app_data::db::DbAppState,
    error_handler::{ApiError, StartupError},
    list::submissions::SubmissionStatus,
    notifications::WebsocketNotification,
    scheduled::{sleep_until_next, startup_schedule},
    schema::{notifications, shifts, submissions},
    shifts::{Shift, ShiftStatus},
};
use actix_web::web;
use chrono::Utc;
use diesel::prelude::*;
use serde::Serialize;
use std::{sync::Arc, time::Duration};
use tokio::{sync::broadcast, task};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct CleanupResult {
    pub deleted_notifications: usize,
    pub released_claimed_submissions: usize,
    pub expired_shifts: usize,
}

pub async fn start_data_cleaner(
    db: Arc<DbAppState>,
    notify_tx: broadcast::Sender<WebsocketNotification>,
) -> Result<(), StartupError> {
    let schedule = startup_schedule("DATA_CLEANER_SCHEDULE")?;

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            if let Err(error) = cleanup(db.clone(), notify_tx.clone()).await {
                tracing::error!("Failed to clean data: {error}");
            }
            sleep_until_next(&schedule).await;
        }
    });
    Ok(())
}

pub async fn cleanup(
    db: Arc<DbAppState>,
    notify_tx: broadcast::Sender<WebsocketNotification>,
) -> Result<CleanupResult, ApiError> {
    let (deleted_notifications, released_claimed_submissions, expired_shifts) =
        web::block(move || {
            let conn = &mut db.connection()?;
            let now = Utc::now();

            conn.transaction::<_, ApiError, _>(|conn| {
                let deleted = diesel::delete(
                    notifications::table
                        .filter(notifications::created_at.lt(now - chrono::Duration::days(30))),
                )
                .execute(conn)?;

                let released = diesel::update(
                    submissions::table
                        .filter(submissions::status.eq(SubmissionStatus::Claimed))
                        .filter(submissions::updated_at.lt(now - chrono::Duration::minutes(120))),
                )
                .set((
                    submissions::status.eq(SubmissionStatus::Pending),
                    submissions::reviewer_id.eq(None::<Uuid>),
                ))
                .execute(conn)?;

                let expired = diesel::update(
                    shifts::table
                        .filter(shifts::status.eq(ShiftStatus::Running))
                        .filter(shifts::end_at.lt(now)),
                )
                .set((
                    shifts::status.eq(ShiftStatus::Expired),
                    shifts::updated_at.eq(now),
                ))
                .get_results::<Shift>(conn)?;
                Ok((deleted, released, expired))
            })
        })
        .await??;

    WebsocketNotification::send(&notify_tx, "SHIFTS_MISSED", &expired_shifts);

    Ok(CleanupResult {
        deleted_notifications,
        released_claimed_submissions,
        expired_shifts: expired_shifts.len(),
    })
}
