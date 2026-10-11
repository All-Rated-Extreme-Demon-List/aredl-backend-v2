use crate::{
    app_data::db::DbAppState,
    error_handler::{ApiError, StartupError},
    notifications::WebsocketNotification,
    scheduled::{sleep_until_next, startup_schedule},
    shifts::RecurringShift,
};
use actix_web::web;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::{sync::broadcast, task};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateShiftsResult {
    pub date: NaiveDate,
    pub created_shifts: usize,
}

pub async fn start_recurrent_shift_creator(
    db: Arc<DbAppState>,
    notify_tx: broadcast::Sender<WebsocketNotification>,
) -> Result<(), StartupError> {
    let schedule = startup_schedule("RECURRING_SHIFTS_SCHEDULE")?;
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let today = Utc::now().date_naive();
            if let Err(error) = create_shifts(db.clone(), notify_tx.clone(), today).await {
                tracing::error!("Failed to create shifts for {today}: {error}");
            }
            sleep_until_next(&schedule).await;
        }
    });
    Ok(())
}

pub async fn create_shifts(
    db: Arc<DbAppState>,
    notify_tx: broadcast::Sender<WebsocketNotification>,
    date: NaiveDate,
) -> Result<CreateShiftsResult, ApiError> {
    let new_shifts =
        web::block(move || RecurringShift::create_shifts(&mut db.connection()?, date)).await??;
    WebsocketNotification::send(&notify_tx, "SHIFTS_CREATED", &new_shifts);
    Ok(CreateShiftsResult {
        date,
        created_shifts: new_shifts.len(),
    })
}
