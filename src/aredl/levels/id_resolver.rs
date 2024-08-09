use std::sync::Arc;
use actix_web::web;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::schema::aredl_levels;


fn resolve_gd_id(db: &web::Data<Arc<DbAppState>>, s: &str) -> Result<Uuid, ApiError> {
    let (parsed_id, two_player) = if s.ends_with("_2p") {
        (s[..s.len() - 3].parse::<i32>(), true)
    } else {
        (s.parse::<i32>(), false)
    };
    let id = parsed_id.map_err(|_| ApiError::new(400, format!("Failed to parse {}", s).as_str()))?;
    let resolved_id = aredl_levels::table
        .filter(aredl_levels::level_id.eq(id))
        .filter(aredl_levels::two_player.eq(two_player))
        .select(aredl_levels::id)
        .first::<Uuid>(&mut db.connection().map_err(|_| ApiError::new(400, format!("Failed to resolve {}", s).as_str()))?)
        .map_err(|_| ApiError::new(400, format!("Failed to resolve {}", s).as_str()))?;
    Ok(resolved_id)
}

pub fn resolve_level_id(db: &web::Data<Arc<DbAppState>>, v: &str) -> Result<Uuid, ApiError> {
    match Uuid::parse_str(v) {
        Ok(uuid) => Ok(uuid),
        Err(_) => resolve_gd_id(db, v)
    }
}