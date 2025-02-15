use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use cron::Schedule;
use diesel::RunQueryDsl;
use tokio::task;
use crate::db::DbAppState;

pub fn start_notifications_cleaner(db: Arc<DbAppState>) {
    let schedule = Schedule::from_str(env::var("NOTIFICATIONS_CLEAN_SCHEDULE").expect("NOTIFICATIONS_CLEAN_SCHEDULE not set").as_str()).unwrap();
    let schedule = Arc::new(schedule);
    let db_clone = db.clone();

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            println!("Removing old notifications");

            let conn_result = db_clone.connection();

            if conn_result.is_err() {
                println!("Failed to clean notifications {}", conn_result.err().unwrap());
                continue;
            }

            let mut conn = conn_result.unwrap();

            let result = diesel::sql_query("DELETE FROM notifications WHERE created_at < NOW() - INTERVAL '1 month'")
                .execute(&mut conn);

            if result.is_err() {
                println!("Failed to clean notifications {}", result.err().unwrap())
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;
        }
    });
}