use crate::{
    aredl::submissions::*,
    auth::{Authenticated, Permission},
    db::DbAppState,
    error_handler::ApiError,
    schema::{
        aredl_submissions, submission_history
    },
};
use actix_web::web;
use diesel::{Connection, ExpressionMethods, RunQueryDsl};
use std::sync::Arc;
use uuid::Uuid;

impl Submission {
    pub fn delete(
        db: web::Data<Arc<DbAppState>>,
        submission_id: Uuid,
        authenticated: Authenticated,
    ) -> Result<(), ApiError> {
        let mut conn = db.connection()?;
        conn.transaction(|connection| -> Result<(), ApiError> {
            let has_auth = authenticated.has_permission(db, Permission::SubmissionReview)?;

            // Log deletion in submission history
            let history = SubmissionHistory {
                id: Uuid::new_v4(),
                submission_id,
                record_id: None,
                status: SubmissionStatus::Denied, // Or SubmissionStatus::Deleted if you add it
                rejection_reason: Some("Submission deleted".into()),
                timestamp: chrono::Utc::now().naive_utc(),
            };
            diesel::insert_into(submission_history::table)
                .values(&history)
                .execute(connection)?;

            let mut query = diesel::delete(aredl_submissions::table)
                .filter(aredl_submissions::id.eq(submission_id))
                .into_boxed();

            if !has_auth {
                query = query
                    .filter(aredl_submissions::submitted_by.eq(authenticated.user_id))
                    .filter(aredl_submissions::status.eq(SubmissionStatus::Pending));
            }

            query.execute(connection)?;
            Ok(())
        })?;
        Ok(())
    }
}
