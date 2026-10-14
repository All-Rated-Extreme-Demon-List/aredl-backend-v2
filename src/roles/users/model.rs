use crate::app_data::db::DbConnection;
use crate::audit::{AuditEntityType, AuditLogEntry};
use crate::auth::Authenticated;
use crate::error_handler::ApiError;
use crate::roles::Role;
use crate::schema::{user_roles, users};
use crate::users::BaseUser;
use diesel::insert_into;
use uuid::Uuid;

use diesel::prelude::*;
impl BaseUser {
    pub fn role_add_all(
        conn: &mut DbConnection,
        role_id: Uuid,
        authenticated: Authenticated,
        users: Vec<Uuid>,
    ) -> Result<Vec<Self>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            let role = Role::user_can_edit(connection, &authenticated, role_id)?;
            let added = Self::add_users(role_id, users.as_ref(), connection)?;
            AuditLogEntry::log_list_change(
                connection,
                Some(authenticated.user_id),
                &role,
                AuditEntityType::RoleUser,
                &[],
                &added,
            )?;

            let users: Vec<BaseUser> = user_roles::table
                .filter(user_roles::role_id.eq(role_id))
                .inner_join(users::table)
                .select(BaseUser::as_select())
                .load(connection)?;
            Ok(users)
        })
    }

    pub fn role_set_all(
        conn: &mut DbConnection,
        role_id: Uuid,
        authenticated: Authenticated,
        users: Vec<Uuid>,
    ) -> Result<Vec<Self>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            let role = Role::user_can_edit(connection, &authenticated, role_id)?;
            let before = diesel::delete(user_roles::table.filter(user_roles::role_id.eq(role_id)))
                .returning(user_roles::user_id)
                .get_results::<Uuid>(connection)?;

            let after = Self::add_users(role_id, &users, connection)?;
            AuditLogEntry::log_list_change(
                connection,
                Some(authenticated.user_id),
                &role,
                AuditEntityType::RoleUser,
                &before,
                &after,
            )?;

            let users: Vec<BaseUser> = user_roles::table
                .filter(user_roles::role_id.eq(role_id))
                .inner_join(users::table)
                .select(BaseUser::as_select())
                .load(connection)?;
            Ok(users)
        })
    }

    pub fn role_delete_all(
        conn: &mut DbConnection,
        role_id: Uuid,
        authenticated: Authenticated,
        users: Vec<Uuid>,
    ) -> Result<Vec<Self>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            let role = Role::user_can_edit(connection, &authenticated, role_id)?;
            let removed = Self::delete_users(role_id, &users, connection)?;
            AuditLogEntry::log_list_change(
                connection,
                Some(authenticated.user_id),
                &role,
                AuditEntityType::RoleUser,
                &removed,
                &[],
            )?;

            let users: Vec<BaseUser> = user_roles::table
                .filter(user_roles::role_id.eq(role_id))
                .inner_join(users::table)
                .select(BaseUser::as_select())
                .load(connection)?;
            Ok(users)
        })
    }

    fn add_users(
        role_id: Uuid,
        users: &[Uuid],
        conn: &mut DbConnection,
    ) -> Result<Vec<Uuid>, ApiError> {
        let added = insert_into(user_roles::table)
            .values(
                users
                    .iter()
                    .map(|user| {
                        (
                            user_roles::user_id.eq(user),
                            user_roles::role_id.eq(role_id),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .returning(user_roles::user_id)
            .get_results(conn)?;
        Ok(added)
    }

    pub fn delete_users(
        role_id: Uuid,
        users: &[Uuid],
        conn: &mut DbConnection,
    ) -> Result<Vec<Uuid>, ApiError> {
        let removed = diesel::delete(
            user_roles::table
                .filter(user_roles::role_id.eq(role_id))
                .filter(user_roles::user_id.eq_any(users)),
        )
        .returning(user_roles::user_id)
        .get_results(conn)?;
        Ok(removed)
    }
}
