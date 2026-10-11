use crate::app_data::db::DbConnection;
use crate::auth::Permission;
use crate::error_handler::ApiError;
use crate::list::submissions::SubmissionStatus;
use crate::schema::{role_permissions_full, submissions, user_roles};
use diesel::prelude::*;
use uuid::Uuid;

pub(crate) fn grant_patreon_plus(
    conn: &mut DbConnection,
    user_id: Uuid,
) -> Result<usize, ApiError> {
    let role_id = patreon_plus_role_id(conn)?;

    diesel::insert_into(user_roles::table)
        .values((
            user_roles::role_id.eq(role_id),
            user_roles::user_id.eq(user_id),
        ))
        .on_conflict_do_nothing()
        .execute(conn)?;

    set_users_submissions_to_priority(conn, &[user_id])
}

pub(crate) fn patreon_plus_role_id(conn: &mut DbConnection) -> Result<i32, ApiError> {
    Ok(role_permissions_full::table
        .select(role_permissions_full::role_id)
        .group_by(role_permissions_full::role_id)
        .having(diesel::dsl::count(role_permissions_full::permission).eq(1))
        .having(
            diesel::dsl::max(role_permissions_full::permission)
                .eq(Permission::SubmissionPriority.to_string()),
        )
        .first::<i32>(conn)?)
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
