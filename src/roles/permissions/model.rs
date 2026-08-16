use crate::app_data::db::DbConnection;
use crate::auth::{Authenticated, Permission};
use crate::error_handler::ApiError;
use crate::roles::Role;
use crate::schema::{role_permissions, role_permissions_full};
use diesel::insert_into;
use std::collections::HashSet;
use std::str::FromStr as _;

use diesel::prelude::*;
impl Role {
    pub fn permission_find_all(
        conn: &mut DbConnection,
        role_id: i32,
        authenticated: &Authenticated,
    ) -> Result<Vec<String>, ApiError> {
        Self::user_can_edit(conn, authenticated, role_id)?;
        Self::direct_permissions(conn, role_id)
    }

    pub fn permission_find_all_resolved(
        conn: &mut DbConnection,
        role_id: i32,
        authenticated: &Authenticated,
    ) -> Result<Vec<String>, ApiError> {
        Self::user_can_edit(conn, authenticated, role_id)?;
        let mut permissions = role_permissions_full::table
            .filter(role_permissions_full::role_id.eq(role_id))
            .select(role_permissions_full::permission)
            .load::<String>(conn)?;
        permissions.sort_unstable();
        Ok(permissions)
    }

    pub fn permission_add_all(
        conn: &mut DbConnection,
        role_id: i32,
        authenticated: Authenticated,
        permissions: Vec<String>,
    ) -> Result<Vec<String>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<String>, ApiError> {
            Self::user_can_edit(connection, &authenticated, role_id)?;
            let permissions = Self::validated_permissions(permissions)?;

            if !permissions.is_empty() {
                insert_into(role_permissions::table)
                    .values(
                        permissions
                            .iter()
                            .map(|permission| {
                                (
                                    role_permissions::role_id.eq(role_id),
                                    role_permissions::permission.eq(permission),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                    .on_conflict((role_permissions::role_id, role_permissions::permission))
                    .do_nothing()
                    .execute(connection)?;
            }

            Self::direct_permissions(connection, role_id)
        })
    }

    pub fn permission_set_all(
        conn: &mut DbConnection,
        role_id: i32,
        authenticated: Authenticated,
        permissions: Vec<String>,
    ) -> Result<Vec<String>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<String>, ApiError> {
            Self::user_can_edit(connection, &authenticated, role_id)?;
            let permissions = Self::validated_permissions(permissions)?;

            diesel::delete(role_permissions::table.filter(role_permissions::role_id.eq(role_id)))
                .execute(connection)?;

            if !permissions.is_empty() {
                insert_into(role_permissions::table)
                    .values(
                        permissions
                            .iter()
                            .map(|permission| {
                                (
                                    role_permissions::role_id.eq(role_id),
                                    role_permissions::permission.eq(permission),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                    .execute(connection)?;
            }

            Self::direct_permissions(connection, role_id)
        })
    }

    pub fn permission_delete_all(
        conn: &mut DbConnection,
        role_id: i32,
        authenticated: Authenticated,
        permissions: Vec<String>,
    ) -> Result<Vec<String>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<String>, ApiError> {
            Self::user_can_edit(connection, &authenticated, role_id)?;
            let permissions = Self::validated_permissions(permissions)?;

            diesel::delete(
                role_permissions::table
                    .filter(role_permissions::role_id.eq(role_id))
                    .filter(role_permissions::permission.eq_any(permissions)),
            )
            .execute(connection)?;

            Self::direct_permissions(connection, role_id)
        })
    }

    fn direct_permissions(conn: &mut DbConnection, role_id: i32) -> Result<Vec<String>, ApiError> {
        let mut permissions = role_permissions::table
            .filter(role_permissions::role_id.eq(role_id))
            .select(role_permissions::permission)
            .load::<String>(conn)?;
        permissions.sort_unstable();
        Ok(permissions)
    }

    fn validated_permissions(permissions: Vec<String>) -> Result<Vec<String>, ApiError> {
        permissions
            .into_iter()
            .map(|permission| {
                Permission::from_str(&permission)
                    .map(|permission| permission.to_string())
                    .map_err(|_error| {
                        ApiError::BadRequest(format!("Unknown permission: {permission}"))
                    })
            })
            .collect::<Result<HashSet<_>, ApiError>>()
            .map(|permissions| {
                let mut permissions = permissions.into_iter().collect::<Vec<_>>();
                permissions.sort_unstable();
                permissions
            })
    }
}
