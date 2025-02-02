use std::sync::Arc;
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use crate::error_handler::ApiError;
use crate::custom_schema::aredl_position_history_full_view;
use crate::db::DbAppState;
use crate::schema::aredl_levels;
use crate::aredl::levels::BaseLevel;

#[derive(Clone, Serialize, Deserialize, ToSchema)]

pub enum HistoryEvent {
    Placed,
    MovedUp,
    MovedDown,
    OtherPlaced,
    OtherRemoved,
    OtherMovedUp,
    OtherMovedDown,
}

impl HistoryEvent {
    pub fn from_history(data: &HistoryLevelFull, level_id: Uuid) -> Self {
        match (level_id == data.cause_id, data.moved, data.pos_diff < Some(0)) {
            (true, true, true) => Self::MovedUp,
            (true, true, false) => Self::MovedDown,
            (true, false, _) => Self::Placed,
            (false, true, false) => Self::OtherMovedUp,
            (false, true, true) => Self::OtherMovedDown,
            (false, false, true) => Self::OtherRemoved,
            (false, false, false) => Self::OtherPlaced,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HistoryLevelResponse {
    /// Position of the level after the action
    pub position: Option<i32>,
    /// Relative difference between the previous and the new position
    pub position_diff: Option<i32>,
    /// The type of event that caused the change
    pub event: HistoryEvent,
    /// Whether the level is now in legacy after the action or not
    pub legacy: bool,
    /// When the action was performed
    pub action_at: NaiveDateTime,
    /// The level that caused the change. Might be another level or the level itself
    pub cause: BaseLevel,
}

impl HistoryLevelResponse {
    pub fn from_data(data: &HistoryLevelFull, level_id: Uuid) -> Self {
        Self {
            position: data.position,
            position_diff: data.pos_diff,
            event: HistoryEvent::from_history(data, level_id),
            legacy: data.legacy,
            action_at: data.action_at,
            cause: BaseLevel {
                id: data.cause_id,
                name: data.cause_name.clone(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct HistoryLevelFull {
    pub position: Option<i32>,
    pub pos_diff: Option<i32>,
    pub moved: bool,
    pub legacy: bool,
    pub action_at: NaiveDateTime,
    pub cause_id: Uuid,
    pub cause_name: String,
}

impl HistoryLevelFull {
    pub fn find(db: web::Data<Arc<DbAppState>>, id: Uuid) -> Result<Vec<Self>, ApiError> {
        let entries = aredl_position_history_full_view::table
            .filter(aredl_position_history_full_view::affected_level.eq(id))
            .inner_join(aredl_levels::table.on(aredl_levels::id.eq(aredl_position_history_full_view::cause)))
            .order_by(aredl_position_history_full_view::ord.desc())
            .select((
                aredl_position_history_full_view::position,
                aredl_position_history_full_view::pos_diff,
                aredl_position_history_full_view::moved,
                aredl_position_history_full_view::legacy,
                aredl_position_history_full_view::action_at,
                aredl_position_history_full_view::cause,
                aredl_levels::name,
            ))
            .load::<HistoryLevelFull>(&mut db.connection()?)?;
        Ok(entries)


    }
}