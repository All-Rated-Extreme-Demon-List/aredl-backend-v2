use crate::aredl::shifts::Shift as AredlShift;
use crate::aredl::shifts::ShiftStatus as AredlShiftStatus;
use crate::arepl::shifts::Shift as AreplShift;
use crate::arepl::shifts::ShiftStatus as AreplShiftStatus;
use crate::db::DbAppState;
use crate::notifications::WebsocketNotification;
use crate::schema::aredl::shifts as aredl_shifts;
use crate::schema::arepl::shifts as arepl_shifts;

use crate::get_secret;
use chrono::Utc;
use cron::Schedule;
use diesel::query_dsl::methods::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task;

pub async fn start_data_cleaner(
    db: Arc<DbAppState>,
    notify_tx: broadcast::Sender<WebsocketNotification>,
) {
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
                "UPDATE aredl.submissions \
                 SET status = 'Pending' \
                 WHERE status = 'Claimed' \
                   AND updated_at < NOW() - INTERVAL '120 minutes';",
            )
            .execute(&mut conn);

            if result.is_err() {
                tracing::error!(
                    "Failed to clean stale submissions claims for AREDL: {}",
                    result.err().unwrap()
                );
            }

            let result = diesel::sql_query(
                "UPDATE arepl.submissions \
                 SET status = 'Pending' \
                 WHERE status = 'Claimed' \
                   AND updated_at < NOW() - INTERVAL '120 minutes';",
            )
            .execute(&mut conn);

            if result.is_err() {
                tracing::error!(
                    "Failed to clean stale submissions claims for AREPL: {}",
                    result.err().unwrap()
                );
            }

            tracing::info!("Expiring overdue shifts");

            let aredl_expired_shifts: Vec<AredlShift> = aredl_shifts::table
                .filter(aredl_shifts::status.eq(AredlShiftStatus::Running))
                .filter(aredl_shifts::end_at.lt(Utc::now()))
                .load(&mut conn)
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to load expired shifts: {}", e);
                    vec![]
                });

            if let Err(e) = diesel::update(
                aredl_shifts::table
                    .filter(aredl_shifts::status.eq(AredlShiftStatus::Running))
                    .filter(aredl_shifts::end_at.lt(Utc::now())),
            )
            .set((
                aredl_shifts::status.eq(AredlShiftStatus::Expired),
                aredl_shifts::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            {
                tracing::error!("Failed to expire AREDL shifts: {}", e);
            }

            let arepl_expired_shifts: Vec<AreplShift> = arepl_shifts::table
                .filter(arepl_shifts::status.eq(AreplShiftStatus::Running))
                .filter(arepl_shifts::end_at.lt(Utc::now()))
                .load(&mut conn)
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to load expired shifts: {}", e);
                    vec![]
                });

            if let Err(e) = diesel::update(
                arepl_shifts::table
                    .filter(arepl_shifts::status.eq(AreplShiftStatus::Running))
                    .filter(arepl_shifts::end_at.lt(Utc::now())),
            )
            .set((
                arepl_shifts::status.eq(AreplShiftStatus::Expired),
                arepl_shifts::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            {
                tracing::error!("Failed to expire AREPL shifts: {}", e);
            }

            let missed_shifts_payload = serde_json::json!({
                "aredl": aredl_expired_shifts,
                "arepl": arepl_expired_shifts,
            });

            let notification = WebsocketNotification {
                notification_type: "SHIFTS_MISSED".into(),
                data: serde_json::to_value(&missed_shifts_payload)
                    .expect("Failed to serialize shifts"),
            };
            if let Err(e) = notify_tx.send(notification) {
                tracing::error!("Failed to send shift notification: {}", e);
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tracing::info!("Cleaned data successfully");

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;
        }
    });
}
