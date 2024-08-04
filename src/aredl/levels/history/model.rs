use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db;
use crate::error_handler::ApiError;
use crate::custom_schema::aredl_position_history_full_view;
use crate::schema::aredl_levels;

#[derive(Serialize, Deserialize, Queryable)]
pub struct HistoryLevelFull {
    pub position: Option<i32>,
    pub moved: bool,
    pub legacy: bool,
    pub action_at: NaiveDateTime,
    pub cause_id: Uuid,
    pub cause_name: String,
}

impl HistoryLevelFull {
    pub fn find(id: Uuid) -> Result<Vec<Self>, ApiError> {
        let entries = aredl_position_history_full_view::table
            .filter(aredl_position_history_full_view::affected_level.eq(id))
            .inner_join(aredl_levels::table.on(aredl_levels::id.eq(aredl_position_history_full_view::cause)))
            .order_by(aredl_position_history_full_view::action_at.desc())
            .select((
                aredl_position_history_full_view::position,
                aredl_position_history_full_view::moved,
                aredl_position_history_full_view::legacy,
                aredl_position_history_full_view::action_at,
                aredl_position_history_full_view::cause,
                aredl_levels::name,
            ))
            .load::<HistoryLevelFull>(&mut db::connection()?)?;
        Ok(entries)


    }
}