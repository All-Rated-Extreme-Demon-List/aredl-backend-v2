use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::list::List;
use crate::schema::levels;
use diesel::pg::Pg;
use std::str::FromStr;
use uuid::Uuid;

use diesel::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelIdentifier {
    Uuid(Uuid),
    /// A GD ID or list position
    Numeric {
        value: i32,
        two_player: bool,
    },
}

impl FromStr for LevelIdentifier {
    type Err = ApiError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Ok(uuid) = Uuid::parse_str(input) {
            return Ok(Self::Uuid(uuid));
        }

        let (number, two_player) = input
            .strip_suffix("_2p")
            .map_or((input, false), |stripped| (stripped, true));
        let value = number
            .parse::<i32>()
            .map_err(|error| ApiError::BadRequest(format!("Failed to parse {input}: {error}")))?;

        Ok(Self::Numeric { value, two_player })
    }
}

impl LevelIdentifier {
    pub fn filter(self, list: List) -> levels::BoxedQuery<'static, Pg> {
        let query = levels::table
            .filter(levels::list_id.eq(list))
            .into_boxed::<Pg>();

        match self {
            Self::Uuid(uuid) => query.filter(levels::id.eq(uuid)),
            Self::Numeric { value, two_player } => query.filter(
                levels::level_id
                    .eq(value)
                    .and(levels::two_player.eq(two_player))
                    .or(levels::position.eq(value)),
            ),
        }
    }
}

pub fn level_filter(list: List, input: &str) -> Result<levels::BoxedQuery<'static, Pg>, ApiError> {
    Ok(input.parse::<LevelIdentifier>()?.filter(list))
}

pub fn resolve_level_id(
    conn: &mut DbConnection,
    list: List,
    level_id: &str,
) -> Result<Uuid, ApiError> {
    level_filter(list, level_id)?
        .select(levels::id)
        .first::<Uuid>(conn)
        .map_err(|error| {
            ApiError::NotFound(
                format!("Failed to resolve {level_id} on list {list}: {error}").as_str(),
            )
        })
}
