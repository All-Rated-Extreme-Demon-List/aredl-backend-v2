use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Integer;
use serde::{Deserialize, Serialize};
use utoipa::openapi::{RefOr, Schema};
use utoipa::{PartialSchema, ToSchema};

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    diesel::AsExpression,
    diesel::FromSqlRow,
)]
#[repr(i32)]
#[diesel(sql_type = Integer)]
#[serde(try_from = "i32", into = "i32")]
pub enum AccountStatus {
    #[default]
    Normal = 0,
    SemiUnranked = 1,
    Unranked = 2,
    ListBanned = 3,
    Redacted = 4,
    Deleted = 5,
}

impl From<AccountStatus> for i32 {
    fn from(account_status: AccountStatus) -> Self {
        account_status as Self
    }
}

impl TryFrom<i32> for AccountStatus {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::SemiUnranked),
            2 => Ok(Self::Unranked),
            3 => Ok(Self::ListBanned),
            4 => Ok(Self::Redacted),
            5 => Ok(Self::Deleted),
            _ => Err(format!("Unknown account status: {value}")),
        }
    }
}

impl ToSql<Integer, Pg> for AccountStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <i32 as ToSql<Integer, Pg>>::to_sql(&i32::from(*self), &mut out.reborrow())
    }
}

impl FromSql<Integer, Pg> for AccountStatus {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        Self::try_from(i32::from_sql(value)?).map_err(Into::into)
    }
}

impl PartialSchema for AccountStatus {
    fn schema() -> RefOr<Schema> {
        i32::schema()
    }
}

impl ToSchema for AccountStatus {}
