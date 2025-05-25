use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::users;
use crate::users::User;
use chrono::{DateTime, Utc};
use diesel::dsl::now;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
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
