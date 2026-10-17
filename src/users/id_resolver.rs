use crate::schema::users;
use diesel::pg::Pg;
use uuid::Uuid;

use diesel::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserIdentifier {
    Uuid(Uuid),
    NameOrDiscord(String),
}

impl From<&str> for UserIdentifier {
    fn from(input: &str) -> Self {
        let input = input.trim_matches('%');
        match Uuid::parse_str(input) {
            Ok(uuid) => Self::Uuid(uuid),
            Err(_) => Self::NameOrDiscord(input.to_owned()),
        }
    }
}

impl From<Uuid> for UserIdentifier {
    fn from(uuid: Uuid) -> Self {
        Self::Uuid(uuid)
    }
}

impl UserIdentifier {
    pub fn filter(self) -> users::BoxedQuery<'static, Pg> {
        let query = users::table.into_boxed::<Pg>();
        match self {
            Self::Uuid(uuid) => query.filter(users::id.eq(uuid)),
            Self::NameOrDiscord(value) => query.filter(
                users::discord_id
                    .eq(Some(value.clone()))
                    .or(users::username.eq(value)),
            ),
        }
    }
}

// user filter that matches either by UUID, username, or discord_id
pub fn user_filter(input: &str) -> users::BoxedQuery<'static, Pg> {
    UserIdentifier::from(input).filter()
}

// user ilike filter that matches either by partial username or global_name or exact user_filter match
pub fn user_ilike_filter(input: &str) -> users::BoxedQuery<'_, Pg> {
    users::table.into_boxed::<Pg>().filter(
        users::username
            .ilike(input)
            .or(users::global_name.ilike(input))
            .or(users::id.eq_any(user_filter(input).select(users::id))),
    )
}
