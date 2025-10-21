use crate::{
    db::DbAppState, get_secret, notifications::WebsocketNotification, shifts::RecurringShift,
};
use chrono::{NaiveDate, Utc};
use cron::Schedule;
use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::{sync::broadcast, task};

pub async fn start_recurrent_shift_creator(
    db: Arc<DbAppState>,
    notify_tx: broadcast::Sender<WebsocketNotification>,
) {
    let schedule = Schedule::from_str(&get_secret("RECURRING_SHIFTS_SCHEDULE")).unwrap();
    let schedule = Arc::new(schedule);

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            tracing::info!("Creating today’s recurring shifts");

            let conn = &mut match db.connection() {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("DB connection failed: {e}");
                    continue;
                }
            };

            let today: NaiveDate = Utc::now().date_naive();
            match RecurringShift::create_shifts(conn, today) {
                Ok(new_shifts) => {
                    let notification = WebsocketNotification {
                        notification_type: "SHIFTS_CREATED".into(),
                        data: serde_json::to_value(&new_shifts)
                            .expect("Failed to serialize shifts"),
                    };
                    if let Err(e) = notify_tx.send(notification) {
                        tracing::error!("Failed to send shift notification: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create shifts for {}: {}", today, e);
                }
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let sleep_secs = (next - now).num_seconds().max(0) as u64;
            tokio::time::sleep(Duration::from_secs(sleep_secs)).await;
        }
    });
}
