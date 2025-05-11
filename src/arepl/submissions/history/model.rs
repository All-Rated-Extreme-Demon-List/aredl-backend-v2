use crate::{
    arepl::submissions::SubmissionStatus,
    auth::{Authenticated, Permission},
    db::DbAppState,
    error_handler::ApiError,
    schema::{arepl::submission_history, users},
    users::BaseUser,
};
use actix_web::web;
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    Selectable, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, ToSchema)]
#[diesel(table_name = submission_history, check_for_backend(Pg))]
pub struct SubmissionHistory {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub record_id: Option<Uuid>,
    pub status: SubmissionStatus,
    pub reviewer_notes: Option<String>,
    pub reviewer_id: Option<Uuid>,
    pub user_notes: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubmissionHistoryResolved {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub record_id: Option<Uuid>,
    pub status: SubmissionStatus,
    pub reviewer_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<BaseUser>,
    pub user_notes: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionHistoryOptions {
    is_record_id: Option<bool>,
}

impl SubmissionHistoryResolved {
    pub fn from_data(history_row: (SubmissionHistory, Option<BaseUser>)) -> Self {
        let (history, user) = history_row;
        Self {
            id: history.id,
            submission_id: history.submission_id,
            record_id: history.record_id,
            status: history.status,
            reviewer_notes: history.reviewer_notes,
            reviewer: user,
            user_notes: history.user_notes,
            timestamp: history.timestamp,
        }
    }

    pub fn by_submission_id(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
        options: SubmissionHistoryOptions,
        authenticated: Authenticated,
    ) -> Result<Vec<SubmissionHistoryResolved>, ApiError> {
        let conn = &mut db.connection()?;

        let mut query = submission_history::table
            .left_join(users::table.on(submission_history::reviewer_id.eq(users::id.nullable())))
            .into_boxed::<Pg>()
            .select((
                SubmissionHistory::as_select(),
                (users::id, users::username, users::global_name).nullable(),
            ))
            .order(submission_history::timestamp.desc());

        if options.is_record_id.unwrap_or(false) {
            query = query.filter(submission_history::record_id.eq(id));
        } else {
            query = query.filter(submission_history::submission_id.eq(id));
        }

        let history_row = query.load::<(SubmissionHistory, Option<BaseUser>)>(conn)?;

        let mut resolved_history = history_row
            .into_iter()
            .map(SubmissionHistoryResolved::from_data)
            .collect::<Vec<_>>();

        if !authenticated.has_permission(db.clone(), Permission::SubmissionReview)? {
            resolved_history.iter_mut().for_each(|h| h.reviewer = None);
        }

        Ok(resolved_history)
    }
}
