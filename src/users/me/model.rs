use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::pg::Pg;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::users;

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub global_name: String,
    pub discord_id: Option<String>,
    pub country: Option<i32>,
    pub discord_avatar: Option<String>,
    pub discord_banner: Option<String>,
    pub discord_accent_color: Option<i32>,
}

impl User {
    pub fn find(conn: &mut DbConnection, id: Uuid) -> Result<Self, ApiError> {
        let user = users::table
            .filter(users::id.eq(id))
            .select(User::as_select())
            .first::<User>(conn)?;
        Ok(user)
    }
}