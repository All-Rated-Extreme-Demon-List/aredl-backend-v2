#[cfg(test)]
use {
    crate::{
        app_data::db::DbAppState,
        audit::{AuditAction, AuditEntityType, AuditLogEntry},
        auth::Permission,
        schema::{
            audit_logs, oauth_connected_accounts, oauth_requests, records, role_permissions, roles,
            user_roles, users,
        },
        users::{AccountStatus, User},
    },
    chrono::{NaiveDateTime, Utc},
    diesel::prelude::*,
    std::sync::Arc,
    uuid::Uuid,
};

#[cfg(test)]
pub const TEST_STAFF_ROLE_PRIVILEGE_LEVEL: i32 = 100;

#[cfg(test)]
pub async fn create_test_user(
    db: &Arc<DbAppState>,
    required_permission: Option<Permission>,
) -> (Uuid, String) {
    let permissions = required_permission.into_iter().collect::<Vec<_>>();
    create_test_user_with_permissions(db, &permissions).await
}

#[cfg(test)]
pub async fn create_test_user_with_permissions(
    db: &Arc<DbAppState>,
    permissions: &[Permission],
) -> (Uuid, String) {
    let conn = &mut db.connection().unwrap();
    let user_id = Uuid::new_v4();
    let username = format!("test_user_{user_id}");

    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_id),
            users::username.eq(&username),
            users::global_name.eq(&username),
            users::discord_id.eq(None::<String>),
            users::placeholder.eq(false),
            users::country.eq(None::<i32>),
            users::discord_avatar.eq(None::<String>),
        ))
        .execute(conn)
        .expect("Failed to create fake user");

    if !permissions.is_empty() {
        let role_id: Uuid = diesel::insert_into(roles::table)
            .values((
                roles::privilege_level.eq(TEST_STAFF_ROLE_PRIVILEGE_LEVEL),
                roles::role_desc.eq("Test Role"),
            ))
            .returning(roles::id)
            .get_result(conn)
            .expect("Failed to create test role");

        diesel::insert_into(user_roles::table)
            .values((
                user_roles::role_id.eq(role_id),
                user_roles::user_id.eq(user_id),
            ))
            .execute(conn)
            .expect("Failed to assign role to user");

        let role_permissions = permissions.iter().map(|permission| {
            (
                role_permissions::role_id.eq(role_id),
                role_permissions::permission.eq(permission.to_string()),
            )
        });

        diesel::insert_into(role_permissions::table)
            .values(role_permissions.collect::<Vec<_>>())
            .execute(conn)
            .expect("Failed to assign permissions to role");
    }

    (user_id, username)
}

#[cfg(test)]
pub async fn create_test_user_with_priority_submissions(db: &Arc<DbAppState>) -> (Uuid, String) {
    create_test_user_with_permissions(db, &[Permission::SubmissionPriority]).await
}

#[cfg(test)]
pub async fn create_test_hidden_reviewer(db: &Arc<DbAppState>) -> (Uuid, String) {
    create_test_user_with_permissions(
        db,
        &[
            Permission::SubmissionReview,
            Permission::SubmissionSeeStatistics,
        ],
    )
    .await
}

#[cfg(test)]
pub async fn create_test_full_reviewer(db: &Arc<DbAppState>) -> (Uuid, String) {
    create_test_user_with_permissions(
        db,
        &[
            Permission::SubmissionReview,
            Permission::SubmissionSeeStatistics,
            Permission::SubmissionReviewerVisible,
            Permission::SubmissionSeeOtherReviewerStatistics,
            Permission::SubmissionEditNonSelfClaimed,
            Permission::SubmissionEditWithRawFootage,
        ],
    )
    .await
}

#[cfg(test)]
pub async fn create_test_auditor(db: &Arc<DbAppState>) -> (Uuid, String) {
    create_test_user_with_permissions(
        db,
        &[
            Permission::SubmissionReview,
            Permission::SubmissionSeeStatistics,
            Permission::SubmissionReviewerVisible,
            Permission::SubmissionSeeOtherReviewerStatistics,
            Permission::SubmissionEditNonSelfClaimed,
            Permission::SubmissionEditWithRawFootage,
            Permission::ReviewersAudit,
        ],
    )
    .await
}

#[cfg(test)]
pub async fn create_test_placeholder_user(db: &Arc<DbAppState>) -> (Uuid, String) {
    let conn = &mut db.connection().unwrap();

    let user_id = Uuid::new_v4();
    let username = format!("test_user_{user_id}");

    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_id),
            users::username.eq(&username),
            users::global_name.eq(&username),
            users::discord_id.eq(None::<String>),
            users::placeholder.eq(true),
            users::country.eq(None::<i32>),
            users::discord_avatar.eq(None::<String>),
        ))
        .execute(conn)
        .expect("Failed to create fake user");

    (user_id, username)
}

#[cfg(test)]
pub async fn set_test_user_country(db: &Arc<DbAppState>, user_id: Uuid, country: Option<i32>) {
    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set(users::country.eq(country))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to assign country to user");
}

#[cfg(test)]
pub async fn set_test_user_account_status(
    db: &Arc<DbAppState>,
    user_id: Uuid,
    account_status: AccountStatus,
) {
    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set(users::account_status.eq(account_status))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set user account status");
}

#[cfg(test)]
pub async fn set_test_user_background_level(
    db: &Arc<DbAppState>,
    user_id: Uuid,
    background_level: i32,
) {
    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set(users::background_level.eq(background_level))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set user background level");
}

#[cfg(test)]
pub async fn set_test_user_discord_id(db: &Arc<DbAppState>, user_id: Uuid, discord_id: &str) {
    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set(users::discord_id.eq(Some(discord_id.to_owned())))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set user discord ID");
}

#[cfg(test)]
pub fn get_test_user(db: &Arc<DbAppState>, user_id: Uuid) -> User {
    users::table
        .filter(users::id.eq(user_id))
        .select(User::as_select())
        .first::<User>(&mut db.connection().unwrap())
        .expect("Failed to read test user")
}

#[cfg(test)]
pub fn set_test_user_discord_profile(db: &Arc<DbAppState>, user_id: Uuid) {
    diesel::update(users::table.find(user_id))
        .set((
            users::discord_id.eq("123456789012345678"),
            users::discord_avatar.eq("avatar_hash"),
            users::discord_avatar_decoration.eq("decoration_asset"),
            users::last_discord_avatar_update.eq(Utc::now().naive_utc()),
            users::description.eq("Personal profile"),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test Discord profile");
}

#[cfg(test)]
pub struct TestUserDataState {
    pub record_count: i64,
    pub connected_account_count: i64,
    pub oauth_request_count: i64,
    pub last_discord_avatar_update: Option<NaiveDateTime>,
}

#[cfg(test)]
pub fn get_test_user_data_state(db: &Arc<DbAppState>, user_id: Uuid) -> TestUserDataState {
    let conn = &mut db.connection().unwrap();
    let record_count: i64 = records::table
        .filter(records::submitted_by.eq(user_id))
        .count()
        .get_result(conn)
        .expect("Failed to count user records");
    let connected_account_count = oauth_connected_accounts::table
        .filter(oauth_connected_accounts::user_id.eq(user_id))
        .count()
        .get_result(conn)
        .expect("Failed to count connected accounts");
    let oauth_request_count = oauth_requests::table
        .filter(oauth_requests::user_id.eq(user_id))
        .count()
        .get_result(conn)
        .expect("Failed to count OAuth requests");
    let last_discord_avatar_update = users::table
        .find(user_id)
        .select(users::last_discord_avatar_update)
        .first(conn)
        .expect("Failed to read avatar update time");
    TestUserDataState {
        record_count,
        connected_account_count,
        oauth_request_count,
        last_discord_avatar_update,
    }
}

#[cfg(test)]
pub fn latest_audit_entry_for_user(
    db: &Arc<DbAppState>,
    user_id: Uuid,
    action: AuditAction,
) -> AuditLogEntry {
    audit_logs::table
        .filter(audit_logs::entity_id.eq(user_id))
        .filter(audit_logs::entity_type.eq(AuditEntityType::User))
        .filter(audit_logs::action_type.eq(action))
        .order(audit_logs::timestamp.desc())
        .select(AuditLogEntry::as_select())
        .first(&mut db.connection().expect("Failed to get DB connection"))
        .expect("Expected audit entry for user")
}
