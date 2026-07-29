use crate::auth::permission;
use crate::auth::Permission;
use crate::error_handler::ApiError;
use crate::schema::{role_permissions, roles, user_roles, users};
use crate::users::BaseUser;
use crate::users::ExtendedBaseUser;
use crate::{app_data::db::DbConnection, auth::Authenticated};
use diesel::{
    ExpressionMethods as _, JoinOnDsl as _, QueryDsl as _, RunQueryDsl as _, SelectableHelper as _,
};
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    Serialize, Deserialize, Queryable, Selectable, Identifiable, PartialEq, Debug, ToSchema,
)]
#[diesel(table_name = roles)]
pub struct Role {
    /// Internal ID of the role.
    pub id: i32,
    /// Privilege level of the role. Refer to [API Overview](#overview) for more information.
    pub privilege_level: i32,
    /// Name of the role.
    pub role_desc: String,
    /// Whether this role should be hidden from public listings and only used to grant permissions.
    pub hide: bool,
    /// Role whose permissions are inherited by this role.
    pub inherits_from_role_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=roles, check_for_backend(Pg))]
pub struct RoleCreate {
    /// Privilege level of the role to create.
    pub privilege_level: i32,
    /// Name of the role to create.
    pub role_desc: String,
    /// Whether this role should be hidden from public listings and only used to grant permissions.
    pub hide: bool,
    /// Role whose permissions are inherited by this role.
    pub inherits_from_role_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=roles, check_for_backend(Pg))]
pub struct RoleUpdate {
    /// New privilege level of the role.
    pub privilege_level: Option<i32>,
    /// New name of the role.
    pub role_desc: Option<String>,
    /// Whether this role should be hidden from public listings and only used to grant permissions.
    pub hide: Option<bool>,
    /// Role whose permissions are inherited by this role.
    pub inherits_from_role_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct RoleResolved {
    #[serde(flatten)]
    pub role: Role,
    /// Users with this role.
    pub users: Vec<BaseUser>,
    /// Permissions directly assigned to this role.
    pub permissions: Vec<String>,
}

impl Role {
    pub fn user_can_edit(
        conn: &mut DbConnection,
        authenticated: &Authenticated,
        target_role_id: i32,
    ) -> Result<(), ApiError> {
        let target_role = roles::table
            .filter(roles::id.eq(target_role_id))
            .first::<Role>(conn)?;

        authenticated
            .has_higher_privilege_than(conn, target_role.privilege_level)
            .then_some(())
            .ok_or_else(|| {
                ApiError::Forbidden("You do not have sufficient permissions to edit this role.")
            })?;

        Ok(())
    }

    pub fn create(
        conn: &mut DbConnection,
        authenticated: &Authenticated,
        role: RoleCreate,
    ) -> Result<Self, ApiError> {
        authenticated
            .has_higher_privilege_than(conn, role.privilege_level)
            .then_some(())
            .ok_or_else(|| {
                ApiError::Forbidden(
                    "You can not create a role with higher permissions than yourself.",
                )
            })?;
        let role = diesel::insert_into(roles::table)
            .values(role)
            .get_result(conn)?;
        Ok(role)
    }

    pub fn update(
        conn: &mut DbConnection,
        authenticated: &Authenticated,
        id: i32,
        role: RoleUpdate,
    ) -> Result<Self, ApiError> {
        Self::user_can_edit(conn, authenticated, id)?;
        let role = diesel::update(roles::table)
            .filter(roles::id.eq(id))
            .set(role)
            .get_result(conn)?;
        Ok(role)
    }

    pub fn delete(
        conn: &mut DbConnection,
        authenticated: &Authenticated,
        id: i32,
    ) -> Result<Self, ApiError> {
        Self::user_can_edit(conn, authenticated, id)?;
        let role = diesel::delete(roles::table)
            .filter(roles::id.eq(id))
            .get_result(conn)?;
        Ok(role)
    }
}

impl RoleResolved {
    pub fn find_all(conn: &mut DbConnection) -> Result<Vec<Self>, ApiError> {
        let roles = roles::table
            .order_by(roles::privilege_level.desc())
            .then_order_by(roles::id.asc())
            .load::<Role>(conn)?;

        let role_ids = roles.iter().map(|role| role.id).collect::<Vec<_>>();

        let users_by_role = user_roles::table
            .inner_join(users::table.on(users::id.eq(user_roles::user_id)))
            .filter(user_roles::role_id.eq_any(&role_ids))
            .select((user_roles::role_id, BaseUser::as_select()))
            .order_by(user_roles::role_id.asc())
            .then_order_by(users::id.asc())
            .load::<(i32, BaseUser)>(conn)?
            .into_iter()
            .into_group_map();

        let permissions_by_role = role_permissions::table
            .filter(role_permissions::role_id.eq_any(&role_ids))
            .select((role_permissions::role_id, role_permissions::permission))
            .order_by(role_permissions::role_id.asc())
            .then_order_by(role_permissions::permission.asc())
            .load::<(i32, String)>(conn)?
            .into_iter()
            .into_group_map();

        Ok(roles
            .into_iter()
            .map(|role| RoleResolved {
                users: users_by_role.get(&role.id).cloned().unwrap_or_default(),
                permissions: permissions_by_role
                    .get(&role.id)
                    .cloned()
                    .unwrap_or_default(),
                role,
            })
            .collect())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ReviewerVisibility {
    /// The ID of the authenticated user.
    pub id: Uuid,
    /// Whether the authenticated user has the permission to audit reviewers.
    pub can_audit: bool,
    /// Whether the authenticated user has the permission to see other reviewers' statistics.
    pub can_see_other_stats: bool,
    /// Whether the authenticated user is a reviewer (shadow or regular)
    pub is_reviewer: bool,
    /// Whether the authenticated user is a hidden reviewer (shadow helper)
    pub is_hidden_reviewer: bool,
    /// Set of hidden reviewers (shadow helpers)
    pub hidden_reviewers: HashSet<Uuid>,
    /// Set of visible reviewers (regular helpers)
    pub visible_reviewers: HashSet<Uuid>,
}

#[derive(Debug, PartialEq)]
pub enum ReviewerFieldVisibility {
    ShowAll,
    HideReviewer,
    HideReviewerAndPrivateNotes,
}

// Utils for handling visibility between shadow and regular helpers.
impl ReviewerVisibility {
    pub fn new(conn: &mut DbConnection, authenticated: &Authenticated) -> Result<Self, ApiError> {
        let can_audit = authenticated.has_permission(conn, Permission::ReviewersAudit)?;
        let is_reviewer = authenticated.has_permission(conn, Permission::SubmissionReview)?;
        let can_see_other_stats =
            authenticated.has_permission(conn, Permission::SubmissionSeeOtherReviewerStatistics)?;

        let reviewers = permission::get_users_with_permission(conn, Permission::SubmissionReview)?;
        let users_with_visible_permission =
            permission::get_users_with_permission(conn, Permission::SubmissionReviewerVisible)?;

        let visible_reviewers = reviewers
            .intersection(&users_with_visible_permission)
            .copied()
            .collect::<HashSet<_>>();
        let hidden_reviewers = reviewers
            .difference(&visible_reviewers)
            .copied()
            .collect::<HashSet<_>>();

        let is_hidden_reviewer = hidden_reviewers.contains(&authenticated.user_id);

        Ok(Self {
            id: authenticated.user_id,
            can_audit,
            can_see_other_stats,
            is_reviewer,
            is_hidden_reviewer,
            hidden_reviewers,
            visible_reviewers,
        })
    }

    // Regular helpers can not see shadow helpers identity, but can see their notes.
    // Shadow helpers can not see regular helpers notes, but can see each others identity/notes.
    // Auditors can see everything.
    pub fn should_hide_reviewer(&self, reviewer_id: Option<&Uuid>) -> ReviewerFieldVisibility {
        if self.can_audit {
            return ReviewerFieldVisibility::ShowAll;
        }

        if !self.is_reviewer {
            return ReviewerFieldVisibility::HideReviewerAndPrivateNotes;
        }

        let Some(reviewer_id) = reviewer_id else {
            return ReviewerFieldVisibility::ShowAll;
        };

        let is_target_hidden = self.hidden_reviewers.contains(reviewer_id);

        match (self.is_hidden_reviewer, is_target_hidden) {
            (false, true) => ReviewerFieldVisibility::HideReviewer,
            (true, false) => ReviewerFieldVisibility::HideReviewerAndPrivateNotes,
            _ => ReviewerFieldVisibility::ShowAll,
        }
    }

    pub fn can_see_identity(&self, reviewer_id: &Uuid) -> bool {
        self.should_hide_reviewer(Some(reviewer_id)) == ReviewerFieldVisibility::ShowAll
    }

    pub fn is_reviewer(&self, reviewer_id: Uuid) -> bool {
        self.hidden_reviewers.contains(&reviewer_id)
            || self.visible_reviewers.contains(&reviewer_id)
    }

    // Shadow helpers can see their own stats, but not others
    // Regular helpers can see other regular helpers stats, but not shadows
    pub fn can_see_stats(&self, reviewer_id: Uuid, always_hide_hidden: bool) -> bool {
        if self.id == reviewer_id {
            return true;
        }

        let is_target_hidden = self.hidden_reviewers.contains(&reviewer_id);

        if always_hide_hidden && is_target_hidden {
            return false;
        }

        if self.can_audit {
            return true;
        }

        !is_target_hidden && self.can_see_other_stats
    }
}

impl ReviewerFieldVisibility {
    pub fn apply_extended(
        self,
        reviewer: &mut Option<ExtendedBaseUser>,
        private_notes: &mut Option<String>,
    ) {
        match self {
            Self::ShowAll => {}
            Self::HideReviewer => {
                *reviewer = reviewer.as_ref().map(|_| ExtendedBaseUser::hidden());
            }
            Self::HideReviewerAndPrivateNotes => {
                *reviewer = reviewer.as_ref().map(|_| ExtendedBaseUser::hidden());
                *private_notes = None;
            }
        }
    }

    pub fn apply_base(self, reviewer: &mut Option<BaseUser>, private_notes: &mut Option<String>) {
        match self {
            Self::ShowAll => {}
            Self::HideReviewer => {
                *reviewer = reviewer.as_ref().map(|_| BaseUser::hidden());
            }
            Self::HideReviewerAndPrivateNotes => {
                *reviewer = reviewer.as_ref().map(|_| BaseUser::hidden());
                *private_notes = None;
            }
        }
    }
}
