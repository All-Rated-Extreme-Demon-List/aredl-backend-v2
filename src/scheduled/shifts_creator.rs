use crate::{aredl::shifts::RecurringShift, db::DbAppState, get_secret};
use chrono::{NaiveDate, Utc};
use cron::Schedule;
use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::task;

pub async fn start_recurrent_shift_creator(db: Arc<DbAppState>) {
    let schedule = Schedule::from_str(&get_secret("RECURRING_SHIFTS_SCHEDULE")).unwrap();
    let schedule = Arc::new(schedule);
    let db_clone = db.clone();

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            tracing::info!("Creating today’s recurring shifts");

            let mut conn = match db_clone.connection() {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("DB connect failed: {}", e);
                    continue;
                }
            };

            let today: NaiveDate = Utc::now().date_naive();
            if let Err(e) = RecurringShift::create_shifts(&mut conn, today) {
                tracing::error!("Failed to create shifts for {}: {}", today, e);
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let sleep_secs = (next - now).num_seconds().max(0) as u64;
            tokio::time::sleep(Duration::from_secs(sleep_secs)).await;
        }
    });
}
