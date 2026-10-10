#[cfg(test)]
use {
    crate::schema::{
        submission_daily_level_stats, submission_daily_reviewer_stats, submission_daily_total_stats,
    },
    crate::{app_data::db::DbAppState, list::List},
    diesel::{prelude::*, sql_query},
    std::sync::Arc,
};

#[cfg(test)]
pub async fn refresh_test_submission_stats(db: &Arc<DbAppState>) {
    sql_query("SELECT rebuild_submission_daily_stats($1)")
        .bind::<diesel::sql_types::SmallInt, _>(List::Classic)
        .execute(&mut db.connection().unwrap())
        .expect("Failed to refresh submission stats");
}

#[cfg(test)]
pub fn clear_test_submission_daily_stats(db: &Arc<DbAppState>) {
    let conn = &mut db.connection().unwrap();
    diesel::delete(
        submission_daily_total_stats::table
            .filter(submission_daily_total_stats::list_id.eq(List::Classic)),
    )
    .execute(conn)
    .expect("Failed to clear total daily stats");
    diesel::delete(
        submission_daily_reviewer_stats::table
            .filter(submission_daily_reviewer_stats::list_id.eq(List::Classic)),
    )
    .execute(conn)
    .expect("Failed to clear reviewer daily stats");
    diesel::delete(
        submission_daily_level_stats::table
            .filter(submission_daily_level_stats::list_id.eq(List::Classic)),
    )
    .execute(conn)
    .expect("Failed to clear level daily stats");
}
