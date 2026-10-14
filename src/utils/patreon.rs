use crate::app_data::db::DbConnection;
use crate::audit::{AuditEntityType, AuditLogEntry};
use crate::auth::Permission;
use crate::error_handler::ApiError;
use crate::list::submissions::SubmissionStatus;
use crate::roles::Role;
use crate::schema::{role_permissions_full, submissions, user_roles};
use diesel::prelude::*;
use uuid::Uuid;

pub(crate) fn grant_patreon_plus(
    conn: &mut DbConnection,
    user_id: Uuid,
) -> Result<usize, ApiError> {
    conn.transaction(|conn| {
        let role_id = patreon_plus_role_id(conn)?;
        let role = Role::find_for_update(conn, role_id)?;

        let added = diesel::insert_into(user_roles::table)
            .values((
                user_roles::role_id.eq(role_id),
                user_roles::user_id.eq(user_id),
            ))
            .on_conflict_do_nothing()
            .returning(user_roles::user_id)
            .get_results::<Uuid>(conn)?;
        AuditLogEntry::log_list_change(conn, None, &role, AuditEntityType::RoleUser, &[], &added)?;

        set_users_submissions_to_priority(conn, &[user_id])
    })
}

pub(crate) fn patreon_plus_role_id(conn: &mut DbConnection) -> Result<Uuid, ApiError> {
    Ok(role_permissions_full::table
        .select(role_permissions_full::role_id)
        .group_by(role_permissions_full::role_id)
        .having(diesel::dsl::count(role_permissions_full::permission).eq(1))
        .having(
            diesel::dsl::max(role_permissions_full::permission)
                .eq(Permission::SubmissionPriority.to_string()),
        )
        .first::<Uuid>(conn)?)
}

pub(crate) fn set_users_submissions_to_priority(
    conn: &mut DbConnection,
    user_ids: &[Uuid],
) -> Result<usize, ApiError> {
    if user_ids.is_empty() {
        return Ok(0);
    }

    let prioritized_count = diesel::update(
        submissions::table
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            .filter(submissions::submitted_by.eq_any(user_ids))
            .filter(submissions::priority.eq(false)),
    )
    .set(submissions::priority.eq(true))
    .execute(conn)?;

    Ok(prioritized_count)
}
