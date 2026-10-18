#[cfg(test)]
use {
    super::{ClanNotifications, UserSettings, UserSettingsUpdate},
    crate::app_data::db::DbAppState,
    std::sync::Arc,
    uuid::Uuid,
};

#[cfg(test)]
pub fn set_test_clan_notifications(
    db: &Arc<DbAppState>,
    user_id: Uuid,
    clan_notifications: ClanNotifications,
) {
    UserSettings::update(
        &mut db.connection().unwrap(),
        user_id,
        &UserSettingsUpdate {
            clan_notifications: Some(clan_notifications),
        },
    )
    .expect("Failed to update test clan notification setting");
}
