use crate::clans::Clan;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{clan_members, clans, permissions, roles, user_roles, users};
use crate::users::{Role, User, UserResolved};
use chrono::{DateTime, Utc};
use diesel::dsl::now;
use diesel::{
    ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, AsChangeset, Debug, ToSchema)]
#[diesel(table_name = users, check_for_backend(Pg))]
pub struct UserMeUpdate {
    /// Your new display name.
    pub global_name: Option<String>,
    /// Your new description.
    pub description: Option<String>,
    /// Your new country. Uses the ISO 3166-1 numeric country code. Has a 90-day cooldown.
    pub country: Option<i32>,
    /// Your new ban level.
    pub ban_level: Option<i32>,
}

impl User {
    pub fn find_me(conn: &mut DbConnection, id: Uuid) -> Result<UserResolved, ApiError> {
        let user = users::table
            .filter(users::id.eq(id))
            .select(User::as_select())
            .first::<User>(conn)?;

        let clan = clans::table
            .inner_join(clan_members::table.on(clans::id.eq(clan_members::clan_id)))
            .filter(clan_members::user_id.eq(id))
            .select(Clan::as_select())
            .first::<Clan>(conn)
            .optional()?;

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
                if user_privilege_level >= privilege_level {
                    Some(permission)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        Ok(UserResolved {
            user,
            clan,
            roles,
            scopes,
        })
    }
    pub fn update_me(
        conn: &mut DbConnection,
        id: Uuid,
        user: UserMeUpdate,
    ) -> Result<User, ApiError> {
        let (current_ban_level, last_country_update): (i32, DateTime<Utc>) = users::table
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
            let current_time = Utc::now();
            if current_time < next_allowed_change {
                let remaining = next_allowed_change - current_time;
                return Err(ApiError::new(400, &format!(
                    "You have recently changed your country, please wait {} days and {} hours before changing it again.",
                    remaining.num_days(), remaining.num_hours() % 24)));
            }
        }

        if user.global_name.is_some() {
            if user.global_name.as_ref().unwrap().len() > 100 {
                return Err(ApiError::new(
                    400,
                    "The display name can at most be 100 characters long.",
                ));
            }
        }

        if user.description.is_some() {
            if user.description.as_ref().unwrap().len() > 300 {
                return Err(ApiError::new(
                    400,
                    "The description can at most be 300 characters long.",
                ));
            }
        }

        let result = diesel::update(users::table.filter(users::id.eq(id)))
            .set((
                &user,
                user.country.map(|_| users::last_country_update.eq(now)),
            ))
            .returning(User::as_select())
            .get_result::<User>(conn)?;
        Ok(result)
    }
}
