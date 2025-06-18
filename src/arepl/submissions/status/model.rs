use crate::{
    db::DbConnection,
    error_handler::ApiError,
    schema::arepl::submissions_enabled,
};
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, ExpressionMethods, RunQueryDsl, Selectable, QueryDsl, SelectableHelper, result::Error as DieselError};
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

    pub fn get_status(
        conn: &mut DbConnection
    ) -> Result<Self, ApiError> {
        let status = submissions_enabled::table
            .order_by(submissions_enabled::created_at.desc())
            .select(Self::as_select())
            .first::<Self>(conn)?;
        Ok(status)
    }

    pub fn get_statuses(
        conn: &mut DbConnection
    ) -> Result<Vec<Self>, ApiError> {
        let status = submissions_enabled::table
            .order_by(submissions_enabled::created_at.desc())
            .select(Self::as_select())
            .load::<Self>(conn)?;
        Ok(status)
    }
}