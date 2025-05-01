use crate::{
    aredl::submissions::SubmissionStatus, db::DbAppState, error_handler::ApiError,
    schema::submission_history,
};
use actix_web::web;
use chrono::NaiveDateTime;
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
    pub rejection_reason: Option<String>,
    pub timestamp: NaiveDateTime,
}

impl SubmissionHistory {
    pub fn by_submission(
        db: web::Data<Arc<DbAppState>>,
        submission_id: Uuid,
    ) -> Result<Vec<SubmissionHistory>, ApiError> {
        let conn = &mut db.connection()?;

        let history = submission_history::table
            .filter(submission_history::submission_id.eq(submission_id))
            .select(SubmissionHistory::as_select())
            .order(submission_history::timestamp.desc())
            .load::<SubmissionHistory>(conn)?;

        Ok(history)
    }
}
