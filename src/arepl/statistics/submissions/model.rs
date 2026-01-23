use crate::app_data::db::DbConnection;
use crate::arepl::levels::ExtendedBaseLevel;
use crate::{
    error_handler::ApiError,
    schema::{arepl::levels, arepl::submission_totals},
};
use diesel::pg::Pg;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema, Clone)]
#[diesel(table_name = submission_totals, check_for_backend(Pg))]
pub struct QueueLevelSubmissionsRow {
    pub level_id: Option<Uuid>,
    pub submissions: i64,
    pub percent_of_queue: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResolvedQueueLevelSubmissionsRow {
    pub level: Option<ExtendedBaseLevel>,
    pub submissions: i64,
    pub percent_of_queue: f64,
}

pub fn total_submissions(
    conn: &mut DbConnection,
) -> Result<Vec<ResolvedQueueLevelSubmissionsRow>, ApiError> {
    let rows: Vec<(QueueLevelSubmissionsRow, Option<ExtendedBaseLevel>)> = submission_totals::table
        .left_join(levels::table.on(levels::id.nullable().eq(submission_totals::level_id)))
        .order_by(submission_totals::submissions.desc())
        .select((
            QueueLevelSubmissionsRow::as_select(),
            Option::<ExtendedBaseLevel>::as_select(),
        ))
        .load(conn)?;

    let resolved = rows
        .into_iter()
        .map(|(stats, level)| ResolvedQueueLevelSubmissionsRow {
            level,
            submissions: stats.submissions,
            percent_of_queue: stats.percent_of_queue,
        })
        .collect::<Vec<_>>();

    Ok(resolved)
}
