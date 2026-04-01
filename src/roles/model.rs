use crate::auth::permission::get_permission_privilege_level;
use crate::auth::Permission;
use crate::error_handler::ApiError;
use crate::schema::{roles, user_roles, users};
use crate::users::BaseUser;
use crate::{app_data::db::DbConnection, auth::Authenticated};
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use itertools::Itertools;
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
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=roles, check_for_backend(Pg))]
pub struct RoleCreate {
    /// Privilege level of the role to create.
    pub privilege_level: i32,
    /// Name of the role to create.
    pub role_desc: String,
    /// Whether this role should be hidden from public listings and only used to grant permissions.
    pub hide: Option<bool>,
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
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct RoleResolved {
    #[serde(flatten)]
    pub role: Role,
    /// Users with this role.
    pub users: Vec<BaseUser>,
}

#[derive(Debug, Default, Clone)]
pub struct ReviewerSets {
    pub base_reviewers: HashSet<Uuid>,
    pub full_reviewers: HashSet<Uuid>,
}

impl Role {
    pub fn user_can_edit(
        conn: &mut DbConnection,
        authenticated: Authenticated,
        target_role_id: i32,
    ) -> Result<(), ApiError> {
        let target_role = roles::table
            .filter(roles::id.eq(target_role_id))
            .first::<Role>(conn)?;

        authenticated
            .has_higher_privilege_than(conn, target_role.privilege_level)?
            .then_some(())
            .ok_or_else(|| {
                ApiError::new(
                    403,
                    "You do not have sufficient permissions to edit this role.".into(),
                )
            })?;

        Ok(())
    }

    pub fn create(
        conn: &mut DbConnection,
        authenticated: Authenticated,
        role: RoleCreate,
    ) -> Result<Self, ApiError> {
        authenticated
            .has_higher_privilege_than(conn, role.privilege_level)?
            .then_some(())
            .ok_or_else(|| {
                ApiError::new(
                    403,
                    "You can not create a role with higher permissions than yourself.".into(),
                )
            })?;
        let role = diesel::insert_into(roles::table)
            .values(role)
            .get_result(conn)?;
        Ok(role)
    }

    pub fn update(
        conn: &mut DbConnection,
        authenticated: Authenticated,
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
        authenticated: Authenticated,
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
        let rows: Vec<(Role, Option<BaseUser>)> = roles::table
            .left_join(user_roles::table.on(user_roles::role_id.eq(roles::id)))
            .left_join(users::table.on(users::id.nullable().eq(user_roles::user_id.nullable())))
            .select((Role::as_select(), Option::<BaseUser>::as_select()))
            .order_by(roles::privilege_level.desc())
            .then_order_by(roles::id.asc())
            .then_order_by(users::id.asc())
            .load(conn)?;

        let resolved = rows
            .into_iter()
            .chunk_by(|(role, _)| role.id)
            .into_iter()
            .map(|(_role_id, group)| {
                let mut role: Option<Role> = None;
                let mut users_vec = Vec::new();

                for (r, u) in group {
                    role.get_or_insert(r);
                    if let Some(u) = u {
                        users_vec.push(u);
                    }
                }

                RoleResolved {
                    role: role.expect("group always has at least one row"),
                    users: users_vec,
                }
            })
            .collect_vec();

        Ok(resolved)
    }

    pub fn find_all_base_reviewers(conn: &mut DbConnection) -> Result<ReviewerSets, ApiError> {
        let base_reviewer_privilege_level =
            get_permission_privilege_level(conn, Permission::SubmissionReviewBase)?;

        let full_reviewer_privilege_level =
            get_permission_privilege_level(conn, Permission::SubmissionReviewFull)?;

        let all_reviewers: Vec<Self> = Self::find_all(conn)?
            .into_iter()
            .filter(|resolved| resolved.role.privilege_level >= base_reviewer_privilege_level)
            .collect();

        let full_reviewers: HashSet<Uuid> = all_reviewers
            .iter()
            .filter(|resolved| resolved.role.privilege_level >= full_reviewer_privilege_level)
            .flat_map(|resolved| resolved.users.iter().map(|user| user.id))
            .collect();

        let base_reviewers: HashSet<Uuid> = all_reviewers
            .iter()
            .filter(|resolved| resolved.role.privilege_level < full_reviewer_privilege_level)
            .flat_map(|resolved| resolved.users.iter().map(|user| user.id))
            .filter(|user_id| !full_reviewers.contains(user_id))
            .collect();

        Ok(ReviewerSets {
            base_reviewers,
            full_reviewers,
        })
    }
}
