use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::arepl::levels;
use diesel::pg::Pg;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

fn parse_gd_id(s: &str) -> Result<(i32, bool), ApiError> {
    let (parsed_id, two_player) = if s.ends_with("_2p") {
        (s[..s.len() - 3].parse::<i32>(), true)
    } else {
        (s.parse::<i32>(), false)
    };

    let id =
        parsed_id.map_err(|_| ApiError::new(400, format!("Failed to parse {}", s).as_str()))?;

    Ok((id, two_player))
}

pub fn level_filter(input: &str) -> Result<levels::BoxedQuery<'static, Pg>, ApiError> {
    let mut query = levels::table.into_boxed::<Pg>();

    match Uuid::parse_str(input) {
        Ok(uuid) => {
            query = query.filter(levels::id.eq(uuid));
        }
        Err(_) => {
            let (id, two_player) = parse_gd_id(input)?;
            query = query
                .filter(levels::level_id.eq(id))
                .filter(levels::two_player.eq(two_player));
        }
    }

    Ok(query)
}

fn resolve_gd_id(conn: &mut DbConnection, s: &str) -> Result<Uuid, ApiError> {
    let (id, two_player) = parse_gd_id(s)?;
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
