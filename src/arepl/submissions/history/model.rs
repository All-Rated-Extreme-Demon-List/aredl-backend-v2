use crate::{
    app_data::db::DbConnection,
    arepl::submissions::SubmissionStatus,
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    schema::{arepl::submission_history, users},
    users::BaseUser,
};
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    Selectable, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, ToSchema)]
#[diesel(table_name = submission_history, check_for_backend(Pg))]
pub struct SubmissionHistory {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub status: SubmissionStatus,
    pub timestamp: DateTime<Utc>,
    pub video_url: Option<String>,
    pub raw_url: Option<String>,
    pub mobile: Option<bool>,
    pub ldm_id: Option<i32>,
    pub mod_menu: Option<String>,
    pub completion_time: Option<i64>,
    pub user_notes: Option<String>,
    pub reviewer_notes: Option<String>,
    pub private_reviewer_notes: Option<String>,
    pub reviewer_id: Option<Uuid>,
    pub locked: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubmissionHistoryResolved {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub status: SubmissionStatus,
    pub timestamp: DateTime<Utc>,
    pub video_url: Option<String>,
    pub raw_url: Option<String>,
    pub mobile: Option<bool>,
    pub ldm_id: Option<i32>,
    pub mod_menu: Option<String>,
    pub completion_time: Option<i64>,
    pub user_notes: Option<String>,
    pub reviewer_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<BaseUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_reviewer_notes: Option<String>,
    pub locked: Option<bool>,
}

impl SubmissionHistoryResolved {
    pub fn from_data(history_row: (SubmissionHistory, Option<BaseUser>)) -> Self {
        let (history, user) = history_row;
        Self {
            id: history.id,
            submission_id: history.submission_id,
            status: history.status,
            reviewer_notes: history.reviewer_notes,
            reviewer: user,
            user_notes: history.user_notes,
            timestamp: history.timestamp,
            video_url: history.video_url,
            raw_url: history.raw_url,
            mobile: history.mobile,
            ldm_id: history.ldm_id,
            mod_menu: history.mod_menu,
            completion_time: history.completion_time,
            private_reviewer_notes: history.private_reviewer_notes,
            locked: history.locked,
        }
    }

    pub fn by_submission_id(
        conn: &mut DbConnection,
        id: Uuid,
        authenticated: Authenticated,
    ) -> Result<Vec<SubmissionHistoryResolved>, ApiError> {
        let history = submission_history::table
            .filter(submission_history::submission_id.eq(id))
            .left_join(users::table.on(submission_history::reviewer_id.eq(users::id.nullable())))
            .into_boxed::<Pg>()
            .select((
                SubmissionHistory::as_select(),
                (users::id, users::username, users::global_name).nullable(),
            ))
            .order(submission_history::timestamp.desc())
            .load::<(SubmissionHistory, Option<BaseUser>)>(conn)?;

        let mut resolved_history = history
            .into_iter()
            .map(SubmissionHistoryResolved::from_data)
            .collect::<Vec<_>>();

        if !authenticated.has_permission(conn, Permission::SubmissionReview)? {
            resolved_history.iter_mut().for_each(|h| {
                h.reviewer = None;
                h.private_reviewer_notes = None;
            });
        }

        Ok(resolved_history)
    }
}
