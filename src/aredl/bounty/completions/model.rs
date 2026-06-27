use chrono::{DateTime, Utc};
use diesel::{
    ExpressionMethods as _, JoinOnDsl as _, QueryDsl as _, RunQueryDsl as _, SelectableHelper as _,
};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app_data::db::DbConnection,
    aredl::bounty::Bounty,
    error_handler::ApiError,
    schema::{
        aredl::{bounty_completed, records},
        users,
    },
    users::ExtendedBaseUser,
};

#[derive(Serialize, Debug, ToSchema)]
pub struct ResolvedCompletedBounty {
    /// The user who completed the bounty.
    pub user: ExtendedBaseUser,
    /// The date this bounty was completed.
    pub completed_at: DateTime<Utc>,
}

impl Bounty {
    pub fn count_completions(self: &Bounty, conn: &mut DbConnection) -> Result<i64, ApiError> {
        let count = bounty_completed::table
            .filter(bounty_completed::bounty_id.eq(self.id))
            .count()
            .get_result(conn)?;
        Ok(count)
    }

    pub fn find_completions_from_id(
        conn: &mut DbConnection,
        bounty_id: Uuid,
    ) -> Result<Vec<ResolvedCompletedBounty>, ApiError> {
        let completions = bounty_completed::table
            .inner_join(users::table.on(users::id.eq(bounty_completed::user_id)))
            .filter(bounty_completed::bounty_id.eq(bounty_id))
            .select((
                ExtendedBaseUser::as_select(),
                bounty_completed::completed_at,
            ))
            .load::<(ExtendedBaseUser, DateTime<Utc>)>(conn)?
            .into_iter()
            .map(|(user, completed_at)| ResolvedCompletedBounty { user, completed_at })
            .collect::<Vec<ResolvedCompletedBounty>>();

        Ok(completions)
    }

    pub fn sync_completions(self, conn: &mut DbConnection) -> Result<(), ApiError> {
        let existing_completions = bounty_completed::table
            .filter(bounty_completed::bounty_id.eq(self.id))
            .select(bounty_completed::user_id)
            .load::<Uuid>(conn)?;

        let existing_count = i64::try_from(existing_completions.len()).map_err(|error| {
            ApiError::InternalServerError(format!(
                "Completion count exceeds supported range: {error}"
            ))
        })?;

        // make sure to stay below the target number if there is one
        let max_missing_completions = self.target_submissions.map_or(i64::MAX, |target| {
            i64::from(target).saturating_sub(existing_count)
        });

        let records = records::table
            .filter(records::level_id.eq(self.level_id))
            .filter(records::achieved_at.ge(self.start_date))
            .filter(records::achieved_at.le(self.end_date.unwrap_or(Utc::now())))
            .filter(records::submitted_by.ne_all(existing_completions))
            .limit(max_missing_completions)
            .select((records::submitted_by, records::achieved_at))
            .load::<(Uuid, DateTime<Utc>)>(conn)?;

        diesel::insert_into(bounty_completed::table)
            .values(
                records
                    .into_iter()
                    .map(|(user_id, achieved_at)| {
                        (
                            bounty_completed::bounty_id.eq(self.id),
                            bounty_completed::user_id.eq(user_id),
                            bounty_completed::completed_at.eq(achieved_at),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .on_conflict((bounty_completed::bounty_id, bounty_completed::user_id))
            .do_nothing()
            .execute(conn)?;

        Ok(())
    }
}
