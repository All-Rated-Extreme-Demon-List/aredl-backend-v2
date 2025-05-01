use crate::{
    aredl::submissions::SubmissionStatus,
    auth::{Authenticated, Permission},
    db::DbAppState,
    error_handler::ApiError,
    schema::submission_history,
};
use actix_web::web;
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_id: Option<Uuid>,
    pub user_notes: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionHistoryOptions {
    is_record_id: Option<bool>,
}

impl SubmissionHistory {
    pub fn by_submission(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
        options: SubmissionHistoryOptions,
        authenticated: Authenticated,
    ) -> Result<Vec<SubmissionHistory>, ApiError> {
        let conn = &mut db.connection()?;

        let mut query = submission_history::table
            .into_boxed::<Pg>()
            .select(SubmissionHistory::as_select())
            .order(submission_history::timestamp.desc());

        if options.is_record_id.unwrap_or(false) {
            query = query.filter(submission_history::record_id.eq(id));
        } else {
            query = query.filter(submission_history::submission_id.eq(id));
        }

        let mut history = query.load::<SubmissionHistory>(conn)?;

        if !authenticated.has_permission(db.clone(), Permission::SubmissionReview)? {
            history.iter_mut().for_each(|h| h.reviewer_id = None);
        }

        Ok(history)
    }
}
