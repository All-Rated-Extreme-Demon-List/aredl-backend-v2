use crate::app_data::db::DbConnection;
use crate::arepl::levels::{BaseLevel, LevelStatus};
use crate::error_handler::ApiError;
use crate::schema::arepl::{levels, position_history_full_view};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, ToSchema)]

pub enum HistoryEvent {
    Pending,
    Placed,
    Removed,
    MovedUp,
    MovedDown,
    OtherPlaced,
    OtherRemoved,
    OtherMovedUp,
    OtherMovedDown,
}

impl HistoryEvent {
    pub fn from_history(data: &HistoryLevelFull, level_id: Uuid) -> Self {
        match (
            data.status.clone(),
            level_id == data.cause_id,
            data.moved,
            data.pos_diff < Some(0),
        ) {
            (LevelStatus::Pending, true, _, _) => Self::Pending,
            (LevelStatus::Removed, true, _, _) => Self::Removed,
            (_, true, true, true) => Self::MovedUp,
            (_, true, true, false) => Self::MovedDown,
            (_, true, false, _) => Self::Placed,
            (_, false, _, false) => Self::OtherPlaced,
            (_, false, _, true) => Self::OtherRemoved,
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
    /// The status of the level after the action.
    pub status: LevelStatus,
    /// When the action was performed
    pub action_at: DateTime<Utc>,
    /// The level that caused the change. Might be another level or the level itself
    pub cause: BaseLevel,
}

impl HistoryLevelResponse {
    pub fn from_data(data: &HistoryLevelFull, level_id: Uuid) -> Self {
        Self {
            position: data.position,
            position_diff: data.pos_diff,
            event: HistoryEvent::from_history(data, level_id),
            status: data.status.clone(),
            action_at: data.action_at,
            cause: BaseLevel {
                id: data.cause_id,
                name: data.cause_name.clone(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct HistoryLevelFull {
    pub position: Option<i32>,
    pub pos_diff: Option<i32>,
    pub moved: bool,
    pub status: LevelStatus,
    pub action_at: DateTime<Utc>,
    pub cause_id: Uuid,
    pub cause_name: String,
}

impl HistoryLevelFull {
    pub fn find(conn: &mut DbConnection, id: Uuid) -> Result<Vec<Self>, ApiError> {
        let entries = position_history_full_view::table
            .filter(position_history_full_view::affected_level.eq(id))
            .inner_join(levels::table.on(levels::id.eq(position_history_full_view::cause)))
            .order_by(position_history_full_view::ord.desc())
            .select((
                position_history_full_view::position,
                position_history_full_view::pos_diff,
                position_history_full_view::moved,
                position_history_full_view::status,
                position_history_full_view::action_at,
                position_history_full_view::cause,
                levels::name,
            ))
            .load::<HistoryLevelFull>(conn)?;
        Ok(entries)
    }
}
