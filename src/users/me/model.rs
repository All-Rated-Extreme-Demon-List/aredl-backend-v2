use crate::error_handler::ApiError;
use crate::schema::{levels, users};
use crate::users::badges::UserBadge;
use crate::users::{AccountStatus, User};
use crate::{app_data::db::DbConnection, schema::records};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use utoipa::ToSchema;

use diesel::prelude::*;
#[derive(Serialize, Deserialize, AsChangeset, Debug, ToSchema)]
#[diesel(table_name = users, check_for_backend(Pg))]
pub struct UserMeUpdate {
    /// Your new display name.
    pub global_name: Option<String>,
    /// Your new description.
    #[serde(default, with = "double_option")]
    pub description: Option<Option<String>>,
    /// Your country. Uses the ISO 3166-1 numeric country code. Can only be set once by the user themselves.
    #[serde(default, with = "double_option")]
    pub country: Option<Option<i32>>,
    /// Your new account status: normal, semi-unranked, or unranked.
    #[schema(minimum = 0, maximum = 2)]
    pub account_status: Option<AccountStatus>,
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
        current_user: &User,
        user: &UserMeUpdate,
    ) -> Result<User, ApiError> {
        let id = current_user.id;

        if current_user.account_status == AccountStatus::Deleted {
            return Err(ApiError::Forbidden("Deleted users cannot be edited."));
        }

        if user.account_status.is_some() && current_user.account_status >= AccountStatus::ListBanned
        {
            return Err(ApiError::Forbidden("You have been banned from the list."));
        }

        if user
            .account_status
            .is_some_and(|account_status| account_status > AccountStatus::Unranked)
        {
            return Err(ApiError::BadRequest(
                "You can only set your account status to normal, semi-unranked, or unranked.",
            ));
        }

        if user.country.is_some() {
            let country = users::table
                .filter(users::id.eq(id))
                .select(users::country)
                .for_update()
                .first::<Option<i32>>(conn)?;
            if country.is_some() {
                return Err(ApiError::BadRequest(
                    "You can only set your country once. Please contact a staff member to change it.",
                ));
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
            let beaten_level: i64 = records::table
                .filter(records::submitted_by.eq(id))
                .inner_join(levels::table)
                .filter(levels::level_id.eq(background_level))
                .count()
                .get_result(conn)?;

            if beaten_level == 0 {
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

        let result = diesel::update(
            users::table
                .filter(users::id.eq(id))
                .filter(users::account_status.ne(AccountStatus::Deleted)),
        )
        .set(user)
        .returning(User::as_select())
        .get_result::<User>(conn)?;
        Ok(result)
    }
}
