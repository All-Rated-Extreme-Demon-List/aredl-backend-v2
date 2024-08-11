use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use cron::Schedule;
use diesel::RunQueryDsl;
use tokio::task;
use crate::db::{DbAppState};

pub fn start_leaderboard_refresher(db: Arc<DbAppState>) {
    let schedule = Schedule::from_str(env::var("LEADERBOARD_REFRESH_SCHEDULE").expect("LEADERBOARD_REFRESH_SCHEDULE not set").as_str()).unwrap();
    let schedule = Arc::new(schedule);
    let db_clone = db.clone();

    task::spawn(async move {
        loop {
            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;

            println!("Refreshing leaderboard");

            let conn = db_clone.connection();

            if conn.is_err() {
                println!("Failed to refresh {}", conn.err().unwrap());
                continue;
            }

            let result = diesel::sql_query("REFRESH MATERIALIZED VIEW aredl_user_leaderboard")
                .execute(&mut conn.unwrap());

            if result.is_err() {
                println!("Failed to refresh {}", result.err().unwrap())
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
}