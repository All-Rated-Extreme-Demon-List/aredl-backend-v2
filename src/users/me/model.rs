use crate::error_handler::ApiError;
use crate::list::levels::Level;
use crate::schema::{levels, users};
use crate::users::badges::UserBadge;
use crate::users::User;
use crate::{app_data::db::DbConnection, schema::records};
use chrono::{DateTime, Utc};
use diesel::dsl::now;
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Serialize, Deserialize, AsChangeset, Debug, ToSchema)]
#[diesel(table_name = users, check_for_backend(Pg))]
pub struct UserMeUpdate {
    /// Your new display name.
    pub global_name: Option<String>,
    /// Your new description.
    #[serde(default, with = "double_option")]
    pub description: Option<Option<String>>,
    /// Your new country. Uses the ISO 3166-1 numeric country code. Has a 90-day cooldown.
    #[serde(default, with = "double_option")]
    pub country: Option<Option<i32>>,
    /// Your new ban level.
    pub ban_level: Option<i32>,
    /// Your new background level. Must be the GD level ID of a level you have beaten. Can be a classic or platformer level. If null, it will be reset to default (uses the hardest beaten level)
    #[serde(default, with = "double_option")]
    pub background_level: Option<Option<i32>>,
    /// Your new featured badge code.
    #[serde(default, with = "double_option")]
    pub featured_badge_code: Option<Option<String>>,
}

impl User {
    pub fn update_me(
        conn: &mut DbConnection,
        id: Uuid,
        user: &UserMeUpdate,
    ) -> Result<User, ApiError> {
        let (current_ban_level, last_country_update): (i32, DateTime<Utc>) = users::table
            .filter(users::id.eq(id))
            .select((users::ban_level, users::last_country_update))
            .first(conn)?;

        if user.ban_level.is_some() && current_ban_level >= 3 {
            return Err(ApiError::Forbidden("You have been banned from the list."));
        }

        if user.country.is_some() {
            let next_allowed_change = last_country_update + chrono::Duration::days(90);
            let current_time = Utc::now();
            if current_time < next_allowed_change {
                let remaining = next_allowed_change - current_time;
                return Err(ApiError::BadRequest(format!(
                    "You have recently changed your country, please wait {} days and {} hours before changing it again.",
                    remaining.num_days(), remaining.num_hours() % 24)));
            }
        }

        if user
            .global_name
            .as_deref()
            .is_some_and(|global_name| global_name.len() > 35)
        {
            return Err(ApiError::BadRequest(
                "The display name can at most be 35 characters long.",
            ));
        }

        if user
            .description
            .as_ref()
            .and_then(|description| description.as_deref())
            .is_some_and(|description| description.len() > 300)
        {
            return Err(ApiError::BadRequest(
                "The description can at most be 300 characters long.",
            ));
        }

        if let Some(Some(background_level)) = user.background_level {
            let beaten_level: Option<Level> = records::table
                .filter(records::submitted_by.eq(id))
                .inner_join(levels::table)
                .filter(levels::level_id.eq(background_level))
                .select(Level::as_select())
                .get_result(conn)
                .optional()?;

            if beaten_level.is_none() {
                return Err(ApiError::BadRequest(
                    "You have not beaten the selected level.",
                ));
            }
        }

        if let Some(Some(code)) = &user.featured_badge_code {
            if !UserBadge::has_code(conn, id, code)? {
                return Err(ApiError::BadRequest(
                    "You have not unlocked the selected badge.",
                ));
            }
        }

        let result = diesel::update(users::table.filter(users::id.eq(id)))
            .set((
                user,
                user.country.map(|_| users::last_country_update.eq(now)),
            ))
            .returning(User::as_select())
            .get_result::<User>(conn)?;
        Ok(result)
    }
}
