use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::pg::Pg;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, JoinOnDsl};
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{users, user_roles, roles, permissions};
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

#[derive(Serialize, Debug)]
pub struct ResolvedUser {
    pub user: User,
    pub roles: Vec<Role>,
    pub scopes: Vec<String>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = roles)]
pub struct Role {
    pub id: i32,
    pub privilege_level: i32,
    pub role_desc: String,
}

impl User {
    pub fn find(conn: &mut DbConnection, id: Uuid) -> Result<ResolvedUser, ApiError> {
        let user = users::table
            .filter(users::id.eq(id))
            .select(User::as_select())
            .first::<User>(conn)?;

        let roles = user_roles::table
            .inner_join(roles::table.on(user_roles::role_id.eq(roles::id)))
            .filter(user_roles::user_id.eq(id))
            .select(Role::as_select())
            .load::<Role>(conn)?;

        let user_privilege_level: i32 = roles
            .iter()
            .map(|role| role.privilege_level)
            .max()
            .unwrap_or(0);

        let all_permissions = permissions::table
            .select((permissions::permission, permissions::privilege_level))
            .load::<(String, i32)>(conn)?;

        let scopes = all_permissions
            .into_iter()
            .filter_map(|(permission, privilege_level)| {
                if user_privilege_level >= privilege_level { Some(permission) } else { None }
            })
            .collect::<Vec<String>>();

        Ok(ResolvedUser { user, roles, scopes })
    }
}