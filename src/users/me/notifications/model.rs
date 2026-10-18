use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::list::List;
use crate::schema::{clan_members, notifications, records, user_settings, users};
use crate::users::me::settings::ClanNotifications;
use chrono::{DateTime, Utc};
use diesel::delete;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::NotificationType"]
#[DbValueStyle = "PascalCase"]
pub enum NotificationType {
    Info,
    Success,
    Failure,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Queryable, Selectable, Insertable)]
#[diesel(table_name = notifications)]
pub struct Notification {
    /// The internal UUID of the notification
    pub id: Uuid,
    /// The internal UUID of the user this notification was sent to
    pub user_id: Uuid,
    /// The content of the notification
    pub content: String,
    /// The type of this notification
    pub notification_type: NotificationType,
    /// Timestamp of when this notification was sent
    pub created_at: DateTime<Utc>,
}

impl Notification {
    pub fn clan_submission_accepted(
        conn: &mut DbConnection,
        list: List,
        submitter_id: Uuid,
        level_id: Uuid,
        level_name: &str,
    ) -> Result<(), ApiError> {
        let clan_id = clan_members::table
            .filter(clan_members::user_id.eq(submitter_id))
            .select(clan_members::clan_id)
            .first::<Uuid>(conn)
            .optional()?;
        let Some(clan_id) = clan_id else {
            return Ok(());
        };

        // someone from the same clan has already beaten this level
        let already_beaten = diesel::select(diesel::dsl::exists(
            records::table
                .inner_join(clan_members::table.on(clan_members::user_id.eq(records::submitted_by)))
                .filter(records::list_id.eq(list))
                .filter(records::level_id.eq(level_id))
                .filter(records::submitted_by.ne(submitter_id))
                .filter(clan_members::clan_id.eq(clan_id)),
        ))
        .get_result::<bool>(conn)?;

        let allowed_settings = if already_beaten {
            vec![ClanNotifications::All]
        } else {
            vec![ClanNotifications::All, ClanNotifications::NewOnly]
        };

        let users_to_send = clan_members::table
            .inner_join(user_settings::table.on(user_settings::user_id.eq(clan_members::user_id)))
            .filter(clan_members::clan_id.eq(clan_id))
            .filter(clan_members::user_id.ne(submitter_id))
            .filter(user_settings::clan_notifications.eq_any(allowed_settings))
            .select(clan_members::user_id)
            .load::<Uuid>(conn)?;

        if users_to_send.is_empty() {
            return Ok(());
        }

        let submitter_name = users::table
            .find(submitter_id)
            .select(users::global_name)
            .first::<String>(conn)?;
        let message =
            format!("{submitter_name}'s submission for {level_name:?} has been accepted!");

        for user_id in users_to_send {
            Self::create(conn, user_id, message.clone(), NotificationType::Success)?;
        }
        Ok(())
    }

    pub fn find_all_me_notifications(
        conn: &mut DbConnection,
        user_id: Uuid,
    ) -> Result<Vec<Notification>, ApiError> {
        let notifications = notifications::table
            .filter(notifications::user_id.eq(user_id))
            .order(notifications::created_at.desc())
            .load::<Notification>(conn)?;
        Ok(notifications)
    }

    pub fn clear_me_notifications(conn: &mut DbConnection, user_id: Uuid) -> Result<(), ApiError> {
        conn.transaction(|connection| -> Result<(), ApiError> {
            delete(notifications::table)
                .filter(notifications::user_id.eq(user_id))
                .execute(connection)?;

            Ok(())
        })?;

        Ok(())
    }

    pub fn create(
        conn: &mut DbConnection,
        user_id: Uuid,
        content: String,
        notification_type: NotificationType,
    ) -> Result<(), ApiError> {
        conn.transaction(|connection| -> Result<(), ApiError> {
            diesel::insert_into(notifications::table)
                .values(Notification {
                    id: Uuid::new_v4(),
                    user_id,
                    content,
                    notification_type,
                    created_at: chrono::Utc::now(),
                })
                .execute(connection)?;

            Ok(())
        })?;

        Ok(())
    }
}
