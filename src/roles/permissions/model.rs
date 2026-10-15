use crate::app_data::db::DbConnection;
use crate::audit::{AuditEntityType, AuditLogEntry, Auditable};
use crate::auth::{Authenticated, Permission};
use crate::error_handler::ApiError;
use crate::roles::Role;
use crate::schema::{role_permissions, role_permissions_full, roles};
use diesel::insert_into;
use serde::Serialize;
use std::collections::HashSet;
use std::str::FromStr as _;

use diesel::prelude::*;
use uuid::Uuid;

#[derive(Serialize)]
struct RolePermission {
    role_id: Uuid,
    permission: String,
}

impl Auditable for RolePermission {
    const ENTITY_TYPE: AuditEntityType = AuditEntityType::Permission;

    fn entity_id(&self) -> Uuid {
        self.role_id
    }

    fn parent_entity_id(&self) -> Option<Uuid> {
        Some(self.role_id)
    }

    fn entity_display(&self, _conn: &mut DbConnection) -> Result<String, ApiError> {
        Ok(self.permission.clone())
    }

    fn parent_display(&self, conn: &mut DbConnection) -> Result<Option<String>, ApiError> {
        Ok(Some(
            roles::table
                .find(self.role_id)
                .select(roles::role_desc)
                .first(conn)?,
        ))
    }

    fn ignored_fields() -> &'static [&'static str] {
        &["role_id"]
    }
}

impl Role {
    pub fn permission_find_all(
        conn: &mut DbConnection,
        role_id: Uuid,
        authenticated: &Authenticated,
    ) -> Result<Vec<String>, ApiError> {
        Self::user_can_edit(conn, authenticated, role_id)?;
        Self::direct_permissions(conn, role_id)
    }

    pub fn permission_find_all_resolved(
        conn: &mut DbConnection,
        role_id: Uuid,
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
        role_id: Uuid,
        authenticated: Authenticated,
        permissions: Vec<String>,
    ) -> Result<Vec<String>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<String>, ApiError> {
            Self::user_can_edit(connection, &authenticated, role_id)?;
            let permissions = Self::validated_permissions(permissions)?;

            if !permissions.is_empty() {
                let added = insert_into(role_permissions::table)
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
                    .returning(role_permissions::permission)
                    .get_results::<String>(connection)?;

                AuditLogEntry::log_membership_changes(
                    connection,
                    Some(authenticated.user_id),
                    &[],
                    &added,
                    |permission| RolePermission {
                        role_id,
                        permission: permission.clone(),
                    },
                )?;
            }

            Self::direct_permissions(connection, role_id)
        })
    }

    pub fn permission_set_all(
        conn: &mut DbConnection,
        role_id: Uuid,
        authenticated: Authenticated,
        permissions: Vec<String>,
    ) -> Result<Vec<String>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<String>, ApiError> {
            Self::user_can_edit(connection, &authenticated, role_id)?;
            let permissions = Self::validated_permissions(permissions)?;

            let before = diesel::delete(
                role_permissions::table.filter(role_permissions::role_id.eq(role_id)),
            )
            .returning(role_permissions::permission)
            .get_results::<String>(connection)?;

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

            let after = Self::direct_permissions(connection, role_id)?;
            AuditLogEntry::log_membership_changes(
                connection,
                Some(authenticated.user_id),
                &before,
                &after,
                |permission| RolePermission {
                    role_id,
                    permission: permission.clone(),
                },
            )?;
            Ok(after)
        })
    }

    pub fn permission_delete_all(
        conn: &mut DbConnection,
        role_id: Uuid,
        authenticated: Authenticated,
        permissions: Vec<String>,
    ) -> Result<Vec<String>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<String>, ApiError> {
            Self::user_can_edit(connection, &authenticated, role_id)?;
            let permissions = Self::validated_permissions(permissions)?;

            let removed = diesel::delete(
                role_permissions::table
                    .filter(role_permissions::role_id.eq(role_id))
                    .filter(role_permissions::permission.eq_any(permissions)),
            )
            .returning(role_permissions::permission)
            .get_results::<String>(connection)?;

            AuditLogEntry::log_membership_changes(
                connection,
                Some(authenticated.user_id),
                &removed,
                &[],
                |permission| RolePermission {
                    role_id,
                    permission: permission.clone(),
                },
            )?;
            Self::direct_permissions(connection, role_id)
        })
    }

    fn direct_permissions(conn: &mut DbConnection, role_id: Uuid) -> Result<Vec<String>, ApiError> {
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
