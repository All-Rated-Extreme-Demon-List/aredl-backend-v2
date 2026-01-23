use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{permissions, roles, user_roles};
use diesel::dsl::max;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Clone, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Permission {
    LevelModify,
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
    ShiftManage,
    NotificationsSubscribe,
}

pub fn get_privilege_level(conn: &mut DbConnection, user_id: Uuid) -> Result<i32, ApiError> {
    let privilege_level: Option<i32> = user_roles::table
        .inner_join(roles::table.on(roles::id.eq(user_roles::role_id)))
        .filter(user_roles::user_id.eq(user_id))
        .select(max(roles::privilege_level))
        .first(conn)
        .unwrap_or(None);
    Ok(privilege_level.unwrap_or(0))
}

pub fn check_user_permission(
    conn: &mut DbConnection,
    user_id: Uuid,
    permission: Permission,
) -> Result<bool, ApiError> {
    let max_privilege = get_privilege_level(conn, user_id)?;
    if max_privilege >= 100 {
        return Ok(true);
    }
    let required_privilege = permissions::table
        .filter(permissions::permission.eq(permission.to_string()))
        .select(permissions::privilege_level)
        .first::<i32>(conn)?;
    Ok(required_privilege <= max_privilege)
}
