use crate::arepl::levels::ExtendedBaseLevel;
use crate::{
    db::DbAppState,
    error_handler::ApiError,
    schema::{arepl::levels, arepl::record_totals},
};
use actix_web::web;
use diesel::pg::Pg;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema, Clone)]
#[diesel(table_name = record_totals, check_for_backend(Pg))]
pub struct LevelTotalRecordsRow {
    pub level_id: Option<Uuid>,
    pub records: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResolvedLevelTotalRecordsRow {
    pub level: Option<ExtendedBaseLevel>,
    pub records: i64,
}

pub fn total_records(
    db: web::Data<Arc<DbAppState>>,
) -> Result<Vec<ResolvedLevelTotalRecordsRow>, ApiError> {
    let conn = &mut db.connection()?;

    let rows: Vec<(LevelTotalRecordsRow, Option<ExtendedBaseLevel>)> = record_totals::table
        .left_join(levels::table.on(levels::id.nullable().eq(record_totals::level_id)))
        .order_by(record_totals::records.desc())
        .select((
            LevelTotalRecordsRow::as_select(),
            Option::<ExtendedBaseLevel>::as_select(),
        ))
        .load(conn)?;

    let resolved = rows
        .into_iter()
        .map(|(stats, level)| ResolvedLevelTotalRecordsRow {
            level,
            records: stats.records,
        })
        .collect::<Vec<_>>();

    Ok(resolved)
}
