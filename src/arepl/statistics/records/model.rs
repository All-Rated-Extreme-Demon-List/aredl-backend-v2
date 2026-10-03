use crate::app_data::db::DbConnection;
use crate::arepl::levels::ExtendedBaseLevel;
use crate::{
    error_handler::ApiError,
    schema::{arepl::levels, arepl::record_totals},
};
use diesel::pg::Pg;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema, Clone)]
#[diesel(table_name = record_totals, check_for_backend(Pg))]
pub struct LevelTotalRecordsRow {
    pub level_id: Option<Uuid>,
    pub records: i64,
    pub verifications: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResolvedLevelTotalRecordsRow {
    /// The level for which these total records are counted (if none, all levels)
    pub level: Option<ExtendedBaseLevel>,
    /// How many accepted records this level currently has
    pub records: i64,
    /// How many verifications this level currently has
    pub verifications: i64,
}

pub fn total_records(
    conn: &mut DbConnection,
) -> Result<Vec<ResolvedLevelTotalRecordsRow>, ApiError> {
    let rows: Vec<(LevelTotalRecordsRow, Option<ExtendedBaseLevel>)> = record_totals::table
        .left_join(levels::table)
        .order_by((
            record_totals::records.desc(),
            record_totals::verifications.desc(),
        ))
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
            verifications: stats.verifications,
        })
        .collect::<Vec<_>>();

    Ok(resolved)
}
