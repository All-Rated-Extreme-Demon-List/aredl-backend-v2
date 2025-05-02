use crate::db::DbAppState;
use crate::get_secret;
use chrono::Utc;
use cron::Schedule;
use diesel::RunQueryDsl;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;

pub fn start_data_cleaner(db: Arc<DbAppState>) {
    let schedule = Schedule::from_str(&get_secret("DATA_CLEANER_SCHEDULE")).unwrap();
    let schedule = Arc::new(schedule);
    let db_clone = db.clone();

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            tracing::info!("Running data cleaner");

            let conn_result = db_clone.connection();

            if conn_result.is_err() {
                tracing::error!("Failed to clean data {}", conn_result.err().unwrap());
                continue;
            }

            let mut conn = conn_result.unwrap();

            tracing::info!("Cleaning old notifications");

            let result = diesel::sql_query(
                "DELETE FROM notifications WHERE created_at < NOW() - INTERVAL '1 month'",
            )
            .execute(&mut conn);

            if result.is_err() {
                tracing::error!("Failed to clean notifications {}", result.err().unwrap())
            }

            tracing::info!("Cleaning stale submissions claims");

            let result = diesel::sql_query(
                "UPDATE aredl_submissions \
                 SET status = 'Pending' \
                 WHERE status = 'Claimed' \
                   AND updated_at < NOW() - INTERVAL '120 minutes';",
            )
            .execute(&mut conn);

            if result.is_err() {
                tracing::error!(
                    "Failed to clean stale submissions claims: {}",
                    result.err().unwrap()
                );
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tracing::info!("Cleaned data successfully");

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;
        }
    });
}
