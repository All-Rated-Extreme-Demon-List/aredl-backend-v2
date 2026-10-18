use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::user_settings;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::ClanNotifications"]
#[DbValueStyle = "PascalCase"]
pub enum ClanNotifications {
    /// No notifications from other clan members
    None,
    /// Get a notification when another clan members completes a level that no one else in the clan has done yet
    NewOnly,
    /// Get a notification when another clan member completes any level
    All,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, ToSchema)]
#[diesel(table_name = user_settings, check_for_backend(Pg))]
pub struct UserSettings {
    /// Internal UUID of the user
    pub user_id: Uuid,
    /// Which clan notifications the user should receive
    #[schema(default = "None")]
    pub clan_notifications: ClanNotifications,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = user_settings, check_for_backend(Pg))]
pub struct UserSettingsUpdate {
    /// Which clan notifications the user should receive
    pub clan_notifications: Option<ClanNotifications>,
}

impl UserSettings {
    pub fn find(conn: &mut DbConnection, user_id: Uuid) -> Result<Self, ApiError> {
        Ok(user_settings::table
            .find(user_id)
            .select(Self::as_select())
            .first(conn)?)
    }

    pub fn update(
        conn: &mut DbConnection,
        user_id: Uuid,
        settings: &UserSettingsUpdate,
    ) -> Result<Self, ApiError> {
        if settings.clan_notifications.is_none() {
            return Err(ApiError::BadRequest("No changes were provided!"));
        }

        Ok(diesel::update(user_settings::table.find(user_id))
            .set(settings)
            .returning(Self::as_select())
            .get_result(conn)?)
    }
}
