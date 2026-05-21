use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app_data::db::DbConnection,
    aredl::bounty::Bounty,
    error_handler::ApiError,
    schema::{aredl::bounty_completed, users},
    users::BaseUser,
};

#[derive(Serialize, Debug, ToSchema)]
pub struct ResolvedCompletedBounty {
    /// The user who completed the bounty.
    pub user: BaseUser,
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
            .select((BaseUser::as_select(), bounty_completed::completed_at))
            .load::<(BaseUser, DateTime<Utc>)>(conn)?
            .into_iter()
            .map(|(user, completed_at)| ResolvedCompletedBounty { user, completed_at })
            .collect::<Vec<ResolvedCompletedBounty>>();

        Ok(completions)
    }
}
