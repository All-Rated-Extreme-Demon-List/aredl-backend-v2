use crate::error_handler::ApiError;
use crate::schema::roles;
use crate::{app_data::db::DbConnection, auth::Authenticated};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
            .has_higher_privilege(conn, target_role.privilege_level)?
            .then_some(())
            .ok_or_else(|| {
                ApiError::new(
                    403,
                    "You do not have sufficient permissions to edit this role.".into(),
                )
            })?;

        Ok(())
    }
    pub fn find_all(conn: &mut DbConnection) -> Result<Vec<Self>, ApiError> {
        let roles = roles::table.load(conn)?;
        Ok(roles)
    }

    pub fn create(
        conn: &mut DbConnection,
        authenticated: Authenticated,
        role: RoleCreate,
    ) -> Result<Self, ApiError> {
        authenticated
            .has_higher_privilege(conn, role.privilege_level)?
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
