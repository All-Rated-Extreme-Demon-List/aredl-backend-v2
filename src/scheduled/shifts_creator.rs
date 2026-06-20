use crate::{
    app_data::db::DbAppState,
    error_handler::StartupError,
    notifications::WebsocketNotification,
    scheduled::{sleep_until_next, startup_schedule},
    shifts::RecurringShift,
};
use chrono::{NaiveDate, Utc};
use std::{sync::Arc, time::Duration};
use tokio::{sync::broadcast, task};

pub async fn start_recurrent_shift_creator(
    db: Arc<DbAppState>,
    notify_tx: broadcast::Sender<WebsocketNotification>,
) -> Result<(), StartupError> {
    let schedule = startup_schedule("RECURRING_SHIFTS_SCHEDULE")?;

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            tracing::info!("Creating today’s recurring shifts");

            let today: NaiveDate = Utc::now().date_naive();
            match db
                .connection()
                .and_then(|mut conn| RecurringShift::create_shifts(&mut conn, today))
            {
                Ok(new_shifts) => {
                    WebsocketNotification::send(&notify_tx, "SHIFTS_CREATED", &new_shifts);
                }
                Err(e) => {
                    tracing::error!("Failed to create shifts for {}: {}", today, e);
                }
            }

            sleep_until_next(&schedule).await;
        }
    });

    Ok(())
}
