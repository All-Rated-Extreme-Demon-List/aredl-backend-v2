use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{role_permissions_full, roles, user_roles};
use diesel::dsl::max;
use diesel::{ExpressionMethods as _, JoinOnDsl as _, QueryDsl as _, RunQueryDsl as _};
use std::collections::HashSet;
use strum_macros::{Display, EnumIter, EnumString};
use uuid::Uuid;

#[derive(Clone, EnumString, Display, EnumIter, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum Permission {
    LevelModify,
    CustomCopiesModify,
    RecordModify,
    PackTierModify,
    PackModify,
    PlaceholderCreate,
    UserModify,
    UserBan,
    UserRedact,
    RoleManage,
    MergeReview,
    DirectMerge,
    ClanModify,
    SubmissionReview,
    SubmissionReviewerVisible,
    SubmissionEditNonSelfClaimed,
    SubmissionEditWithRawFootage,
    SubmissionSeeOtherReviewerStatistics,
    SubmissionPriority,
    ShiftManage,
    SubmissionStatusManage,
    ReviewersAudit,
    NotificationsSubscribe,
    ExternalConnectionsManage,
    BountyManage,
    ShiftCreateOwn,
}

pub fn get_highest_role_privilege_level(conn: &mut DbConnection, user_id: Uuid) -> i32 {
    let privilege_level: Option<i32> = user_roles::table
        .inner_join(roles::table.on(roles::id.eq(user_roles::role_id)))
        .filter(user_roles::user_id.eq(user_id))
        .select(max(roles::privilege_level))
        .first(conn)
        .unwrap_or(None);
    privilege_level.unwrap_or(0)
}

pub fn get_user_permissions(
    conn: &mut DbConnection,
    user_id: Uuid,
    exclude_hidden_roles: bool,
) -> Result<Vec<String>, ApiError> {
    let roles = user_roles::table
        .inner_join(roles::table.on(roles::id.eq(user_roles::role_id)))
        .filter(user_roles::user_id.eq(user_id))
        .select((roles::id, roles::hide))
        .load::<(i32, bool)>(conn)?
        .into_iter()
        .filter(|(_, hide)| !exclude_hidden_roles || !hide)
        .map(|(role_id, _)| role_id)
        .collect::<Vec<i32>>();

    let mut scopes = role_permissions_full::table
        .filter(role_permissions_full::role_id.eq_any(roles))
        .select(role_permissions_full::permission)
        .distinct()
        .load::<String>(conn)?;

    scopes.sort_unstable();
    Ok(scopes)
}

pub fn get_users_with_permission(
    conn: &mut DbConnection,
    permission: Permission,
) -> Result<HashSet<Uuid>, ApiError> {
    Ok(role_permissions_full::table
        .inner_join(user_roles::table.on(user_roles::role_id.eq(role_permissions_full::role_id)))
        .filter(role_permissions_full::permission.eq(permission.to_string()))
        .select(user_roles::user_id)
        .distinct()
        .load::<Uuid>(conn)?
        .into_iter()
        .collect())
}

pub fn check_user_permission(
    conn: &mut DbConnection,
    user_id: Uuid,
    permission: Permission,
) -> Result<bool, ApiError> {
    let user_permissions = get_user_permissions(conn, user_id, false)?;
    Ok(user_permissions.contains(&permission.to_string()))
}
