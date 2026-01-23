use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::arepl::levels;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

fn resolve_gd_id(conn: &mut DbConnection, s: &str) -> Result<Uuid, ApiError> {
    let (parsed_id, two_player) = if s.ends_with("_2p") {
        (s[..s.len() - 3].parse::<i32>(), true)
    } else {
        (s.parse::<i32>(), false)
    };
    let id =
        parsed_id.map_err(|_| ApiError::new(400, format!("Failed to parse {}", s).as_str()))?;
    let resolved_id = levels::table
        .filter(levels::level_id.eq(id))
        .filter(levels::two_player.eq(two_player))
        .select(levels::id)
        .first::<Uuid>(conn)
        .map_err(|_| ApiError::new(404, format!("Failed to resolve {}", s).as_str()))?;

    Ok(resolved_id)
}

pub fn resolve_level_id(conn: &mut DbConnection, v: &str) -> Result<Uuid, ApiError> {
    match Uuid::parse_str(v) {
        Ok(uuid) => Ok(uuid),
        Err(_) => resolve_gd_id(conn, v),
    }
}
