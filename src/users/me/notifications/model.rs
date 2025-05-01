use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::notifications;
use chrono::{DateTime, Utc};
use diesel::{delete, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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
    pub fn find_all_me_notifications(
        conn: &mut DbConnection,
        user_id: Uuid,
    ) -> Result<Vec<Notification>, ApiError> {
        let notifications = notifications::table
            .filter(notifications::user_id.eq(user_id))
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
