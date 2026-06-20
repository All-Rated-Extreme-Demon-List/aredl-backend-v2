use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::arepl::levels;
use diesel::pg::Pg;
use diesel::{ExpressionMethods as _, QueryDsl as _, RunQueryDsl as _};
use uuid::Uuid;

fn parse_gd_id(string: &str) -> Result<(i32, bool), ApiError> {
    let (parsed_id, two_player) = if let Some(stripped) = string.strip_suffix("_2p") {
        (stripped.parse::<i32>(), true)
    } else {
        (string.parse::<i32>(), false)
    };

    let id = parsed_id.map_err(|error| {
        ApiError::BadRequest(format!("Failed to parse {string}: {error}").as_str())
    })?;

    Ok((id, two_player))
}

pub fn level_filter(input: &str) -> Result<levels::BoxedQuery<'static, Pg>, ApiError> {
    let mut query = levels::table.into_boxed::<Pg>();

    if let Ok(uuid) = Uuid::parse_str(input) {
        query = query.filter(levels::id.eq(uuid));
    } else {
        let (id, two_player) = parse_gd_id(input)?;
        query = query
            .filter(levels::level_id.eq(id))
            .filter(levels::two_player.eq(two_player));
    }

    Ok(query)
}

fn resolve_gd_id(conn: &mut DbConnection, string: &str) -> Result<Uuid, ApiError> {
    let (id, two_player) = parse_gd_id(string)?;
    let resolved_id = levels::table
        .filter(levels::level_id.eq(id))
        .filter(levels::two_player.eq(two_player))
        .select(levels::id)
        .first::<Uuid>(conn)
        .map_err(|error| {
            ApiError::NotFound(format!("Failed to resolve {string}: {error}").as_str())
        })?;

    Ok(resolved_id)
}

pub fn resolve_level_id(conn: &mut DbConnection, v: &str) -> Result<Uuid, ApiError> {
    match Uuid::parse_str(v) {
        Ok(uuid) => Ok(uuid),
        Err(_) => resolve_gd_id(conn, v),
    }
}
