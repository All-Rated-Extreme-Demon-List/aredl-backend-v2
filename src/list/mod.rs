use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::SmallInt;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use utoipa::ToSchema;

pub mod bounty;
mod changelog;
mod clan;
mod country;
pub mod leaderboard;
pub mod levels;
mod packs;
mod packtiers;
mod profile;
pub(crate) mod records;
mod routes;
mod statistics;

pub mod submissions;
pub use routes::{init_routes, ApiDoc};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    diesel::AsExpression,
    diesel::FromSqlRow,
    ToSchema,
    Display,
)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "lowercase")]
#[repr(i16)]
/// The selected list: `classic` (or `aredl`)
/// or `platformer` (or `arepl`).
pub enum List {
    #[serde(alias = "aredl")]
    Classic = 1,
    #[serde(alias = "arepl")]
    Platformer = 2,
}

impl From<List> for i16 {
    fn from(list: List) -> Self {
        list as Self
    }
}

impl ToSql<SmallInt, Pg> for List {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <i16 as ToSql<SmallInt, Pg>>::to_sql(&i16::from(*self), &mut out.reborrow())
    }
}

impl FromSql<SmallInt, Pg> for List {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        match i16::from_sql(value)? {
            1 => Ok(Self::Classic),
            2 => Ok(Self::Platformer),
            id => Err(format!("Unknown list ID: {id}").into()),
        }
    }
}
