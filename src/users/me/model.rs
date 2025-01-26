use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, NaiveDateTime};
use diesel::pg::Pg;
use diesel::dsl::now;
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

#[derive(Serialize, Deserialize, AsChangeset, Debug)]
#[diesel(table_name = users, check_for_backend(Pg))]
pub struct UpdateUser {
    pub global_name: Option<String>,
    pub description: Option<String>,
    pub country: Option<i32>,
    pub ban_level: Option<i32>,
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
    pub fn update(conn: &mut DbConnection, id: Uuid, user: UpdateUser) -> Result<(), ApiError> {
        let (current_ban_level, last_country_update): (i32, NaiveDateTime) = users::table
            .filter(users::id.eq(id))
            .select((users::ban_level, users::last_country_update))
            .first(conn)?;

        if user.ban_level.is_some() {
            if current_ban_level > 1 {
                return Err(ApiError::new(403, "You have been banned from the list."));
            }
        }

        if user.country.is_some() {
            let next_allowed_change = last_country_update + chrono::Duration::days(90);
            let current_time = Utc::now().naive_utc();
            if current_time < next_allowed_change {
                let remaining = next_allowed_change - current_time;
                return Err(ApiError::new(400, &format!(
                    "You have recently changed your country, please wait {} days and {} hours before changing it again.",
                    remaining.num_days(), remaining.num_hours() % 24)));
            }
        }

        diesel::update(users::table.filter(users::id.eq(id)))
            .set((
                &user,
                user.country.map(|_| users::last_country_update.eq(now)),
            ))
            .execute(conn)?;
        Ok(())
    }
}