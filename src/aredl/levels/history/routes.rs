use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::aredl::levels::history::HistoryLevelFull;
use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[get("")]
async fn find(db: web::Data<Arc<DbAppState>>, id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, id.into_inner().as_str())?;
    let entries = web::block(move || HistoryLevelFull::find(db, level_id)).await??;
    // map history
    let mut prev_position: Option<i32> = None;
    let response = entries
        .into_iter()
        .map(|data| {
            let result = HistoryLevelResponse::from_data(&data, prev_position, level_id);
            prev_position = data.position;
            result
        }).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(response))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{id}/history")
            .service(find)
    );
}

#[derive(Clone, Serialize, Deserialize)]
pub enum HistoryEvent {
    Placed,
    MovedUp,
    MovedDown,
    OtherPlaced,
    OtherRemoved,
    OtherMoved,
}

impl HistoryEvent {
    pub fn from_history(data: &HistoryLevelFull, prev_position: Option<i32>, level_id: Uuid) -> Self {
        match (level_id == data.cause_id, data.moved, data.position < prev_position) {
            (true, true, true) => Self::MovedUp,
            (true, true, false) => Self::MovedDown,
            (true, false, _) => Self::Placed,
            (false, true, _) => Self::OtherMoved,
            (false, false, true) => Self::OtherRemoved,
            (false, false, false) => Self::OtherPlaced,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HistoryLevel {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct HistoryLevelResponse {
    pub position: Option<i32>,
    pub position_diff: i32,
    pub event: HistoryEvent,
    pub legacy: bool,
    pub action_at: NaiveDateTime,
    pub cause: HistoryLevel,
}

impl HistoryLevelResponse {
    pub fn from_data(data: &HistoryLevelFull, prev_position: Option<i32>, level_id: Uuid) -> Self {
        Self {
            position: data.position,
            position_diff: data.position.unwrap_or(0) - prev_position.unwrap_or(0),
            event: HistoryEvent::from_history(data, prev_position, level_id),
            legacy: data.legacy,
            action_at: data.action_at,
            cause: HistoryLevel {
                id: data.cause_id,
                name: data.cause_name.clone(),
            }
        }
    }
}