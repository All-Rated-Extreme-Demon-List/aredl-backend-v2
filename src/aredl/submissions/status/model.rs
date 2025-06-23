use crate::{
    db::DbConnection,
    error_handler::ApiError,
    schema::{aredl::submissions_enabled, users},
    users::BaseUser
};
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, ExpressionMethods, RunQueryDsl, Selectable, QueryDsl, SelectableHelper, result::Error as DieselError, JoinOnDsl};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Insertable, Selectable, Debug, ToSchema, Clone)]
#[diesel(table_name = submissions_enabled, check_for_backend(Pg))]
pub struct SubmissionsEnabled {
    /// Whether submissions have been enabled or disabled.
    enabled: bool,
    /// The moderator that performed this change
    moderator: Uuid,
    /// Timestamp of when submissions were toggled on or off
    created_at: DateTime<Utc>
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionsEnabledFull {
    /// Whether submissions have been enabled or disabled.
    enabled: bool,
    /// The moderator that performed this change
    moderator: BaseUser,
    /// Timestamp of when submissions were toggled on or off
    created_at: DateTime<Utc>
}

impl SubmissionsEnabled {
    pub fn enable(
        conn: &mut DbConnection,
        user_id: Uuid
    ) -> Result<(), ApiError> {
        diesel::insert_into(submissions_enabled::table)
            .values((
                submissions_enabled::enabled.eq(true),
                submissions_enabled::moderator.eq(user_id)
            ))
            .execute(conn)?;
        Ok(())
    }

    pub fn disable(
        conn: &mut DbConnection,
        user_id: Uuid
    ) -> Result<(), ApiError> {
        diesel::insert_into(submissions_enabled::table)
            .values((
                submissions_enabled::enabled.eq(false),
                submissions_enabled::moderator.eq(user_id)
            ))
            .execute(conn)?;
        Ok(())
    }

    pub fn is_enabled(
        conn: &mut DbConnection
    ) -> Result<bool, ApiError> {
        let status = submissions_enabled::table
            .order_by(submissions_enabled::created_at.desc())
            .select(submissions_enabled::enabled)
            .first::<bool>(conn);
        
        // If submissions have never been disabled before, there will be no rows
        // returned from the table. In this case, submissions are assumed to be
        // enabled by default.
        Ok(match status {
            Ok(status) => status,
            Err(DieselError::NotFound) => true,
            Err(e) => return Err(ApiError::from(e))
        })
    }
}

impl SubmissionsEnabledFull {
    pub fn get_status(
        conn: &mut DbConnection
    ) -> Result<Self, ApiError> {
        let status = submissions_enabled::table
            .order_by(submissions_enabled::created_at.desc())
            .select(SubmissionsEnabled::as_select())
            .first::<SubmissionsEnabled>(conn)?;

        let moderator = users::table
            .filter(users::id.eq(status.moderator))
            .select(BaseUser::as_select())
            .get_result(conn)?;

        Ok(Self {
            enabled: status.enabled,
            moderator,
            created_at: status.created_at,
        })
    }

    pub fn get_statuses(
        conn: &mut DbConnection
    ) -> Result<Vec<Self>, ApiError> {
        let status = submissions_enabled::table
            .order_by(submissions_enabled::created_at.desc())
            .inner_join(users::table.on(users::id.eq(submissions_enabled::moderator)))
            .select((SubmissionsEnabled::as_select(), BaseUser::as_select()))
            .load::<(SubmissionsEnabled, BaseUser)>(conn)?;

        Ok(status
            .into_iter()
            .map(|(status, user)| 
                SubmissionsEnabledFull {
                    enabled: status.enabled,
                    moderator: user,
                    created_at: status.created_at
                })
            .collect::<Vec<SubmissionsEnabledFull>>()
        )
    }
}