#[cfg(test)]
use {
    crate::{
        app_data::db::DbAppState,
        schema::notifications,
        users::me::notifications::{Notification, NotificationType},
    },
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    std::sync::Arc,
    uuid::Uuid,
};

#[cfg(test)]
pub fn create_test_notification(
    db: &Arc<DbAppState>,
    user_id: Uuid,
    message: &str,
    notification_type: NotificationType,
) {
    Notification::create(
        &mut db.connection().unwrap(),
        user_id,
        message.to_string(),
        notification_type,
    )
    .expect("Failed to create test notification");
}

#[cfg(test)]
pub fn count_test_notifications(db: &Arc<DbAppState>, user_id: Uuid) -> i64 {
    notifications::table
        .filter(notifications::user_id.eq(user_id))
        .count()
        .get_result(&mut db.connection().unwrap())
        .expect("Failed to count test notifications")
}
