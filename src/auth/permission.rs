use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{role_permissions_full, roles, user_roles};
use diesel::dsl::max;
use std::collections::HashSet;
use strum_macros::{Display, EnumIter, EnumString};
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Clone, EnumString, Display, EnumIter, Copy)]
#[strum(serialize_all = "snake_case")]
/// Represents all available permissions. "Editing" usually refers to the ability to create, update, and delete.
pub enum Permission {
    /// Allows placing and editing levels on the list
    LevelModify,
    /// Allows editing custom copy IDs for a level
    LevelCustomCopiesModify,
    /// Allows editing updates for a level (nerfs, buffs, balances, etc.)
    LevelUpdatesModify,
    /// Allows editing public or reviewer notes for a level
    LevelNotesModify,
    /// Allows editing existing records for a level
    RecordModify,
    /// Allows editing pack tiers
    PackTierModify,
    /// Allows editing pack and their levels
    PackModify,
    /// Allows creating an empty placeholder user account
    PlaceholderCreate,
    /// Allows editing user names, country, etc
    UserModify,
    /// Allows banning a user (ban_level 2)
    UserBan,
    /// Allows redacting a user (ban_level 3)
    UserRedact,
    /// Allows editing roles and permissions below your highest role
    RoleManage,
    /// Allows reviewing merge requests submitted through the site
    MergeReview,
    /// Allows directly merging two users
    DirectMerge,
    /// Allows editing clans and their members
    ClanModify,
    /// Allows reviewing submissions
    SubmissionReview,
    /// By default, submission reviewers are hidden. This permission makes a reviewer visible to other reviewers.
    SubmissionReviewerVisible,
    /// Allows editing submissions that you do not have claimed through the claim system
    SubmissionEditNonSelfClaimed,
    /// Allows editing submissions that have raw footage attached
    SubmissionEditWithRawFootage,
    /// Allows seeing submission statistics (total and own)
    SubmissionSeeStatistics,
    /// Allows seeing other reviewers' statistics. Otherwise, you can only see your own.
    SubmissionSeeOtherReviewerStatistics,
    /// Sets the user's submissions to priority if they have this. (AREDL+)
    SubmissionPriority,
    /// Allows editing reviewers shifts
    ShiftManage,
    /// Allows editing your own shifts
    ShiftCreateOwn,
    /// Allows opening and closing new submissions.
    SubmissionStatusManage,
    /// Allows seeing the identity of all reviewers, including hidden ones.
    ReviewersAudit,
    /// Allows subscribing to a websocket for submissions notifications. (Used by the discord bot)
    NotificationsSubscribe,
    /// Allows editing users external connections (Patreon)
    ExternalConnectionsManage,
    /// Allows editing weekly, monthly, event and bounty levels
    BountyManage,
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
    let has_permission = role_permissions_full::table
        .inner_join(user_roles::table.on(user_roles::role_id.eq(role_permissions_full::role_id)))
        .filter(user_roles::user_id.eq(user_id))
        .filter(role_permissions_full::permission.eq(permission.to_string()))
        .select(role_permissions_full::permission)
        .first::<String>(conn)
        .optional()?
        .is_some();

    Ok(has_permission)
}
