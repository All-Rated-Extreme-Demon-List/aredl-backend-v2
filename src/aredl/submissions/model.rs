use crate::{
    aredl::levels::ExtendedBaseLevel,
    auth::{Authenticated, Permission},
    db::DbAppState,
    error_handler::ApiError,
    schema::aredl::{submission_history, submissions, submissions_with_priority},
    users::BaseUser,
};
use actix_web::web;
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, Connection, ExpressionMethods, RunQueryDsl, Selectable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use super::history::SubmissionHistory;

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::aredl::sql_types::SubmissionStatus"]
#[DbValueStyle = "PascalCase"]
pub enum SubmissionStatus {
    Pending,
    Claimed,
    UnderConsideration,
    Denied,
    Accepted,
}

#[derive(Serialize, Deserialize)]
pub struct BaseSubmission {
    /// Internal UUID of the submission.
    pub id: Uuid,
    /// Name of the level this submission is for.
    pub level: String,
    /// The submitter's name
    pub submitter: String,
    /// The status of this submission
    pub status: SubmissionStatus,
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Selectable, Debug, ToSchema, Clone)]
#[diesel(table_name = submissions, check_for_backend(Pg))]
pub struct Submission {
    /// Internal UUID of the submission.
    pub id: Uuid,
    /// UUID of the level this record is on.)
    pub level_id: Uuid,
    /// Internal UUID of the submitter.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// The status of this submission
    pub status: SubmissionStatus,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Whether the record was submitted as a priority record.
    pub priority: bool,
    /// Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
    /// Timestamp of when the submission was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the submission was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Selectable, Debug, ToSchema)]
#[diesel(table_name = submissions_with_priority, check_for_backend(Pg))]
pub struct SubmissionWithPriority {
    /// Internal UUID of the submission.
    pub id: Uuid,
    /// UUID of the level this record is on.)
    pub level_id: Uuid,
    /// Internal UUID of the submitter.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// The status of this submission
    pub status: SubmissionStatus,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Whether the record was submitted as a priority record.
    pub priority: bool,
    /// Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
    /// Timestamp of when the submission was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the submission was last updated.
    pub updated_at: DateTime<Utc>,
    /// The priority value of this submission
    pub priority_value: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionResolved {
    /// Internal UUID of the submission.
    pub id: Uuid,
    /// The level this submission is for
    pub level: ExtendedBaseLevel,
    /// User who submitted this completion.
    pub submitted_by: BaseUser,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// The status of this submission
    pub status: SubmissionStatus,
    /// User who reviewed the record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<BaseUser>,
    /// Whether the record was submitted as a priority record.
    pub priority: bool,
    /// Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
    /// Timestamp of when the submission was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the submission was last updated.
    pub updated_at: DateTime<Utc>,
    ///
    pub priority_value: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionPage {
    data: Vec<Submission>,
}

impl Submission {
    pub fn delete(
        db: web::Data<Arc<DbAppState>>,
        submission_id: Uuid,
        authenticated: Authenticated,
    ) -> Result<(), ApiError> {
        let mut conn = db.connection()?;
        conn.transaction(|connection| -> Result<(), ApiError> {
            // Log deletion in submission history
            let history = SubmissionHistory {
                id: Uuid::new_v4(),
                submission_id,
                record_id: None,
                status: SubmissionStatus::Denied, // Or SubmissionStatus::Deleted if you add it
                reviewer_notes: None,
                reviewer_id: None,
                user_notes: Some("Submission deleted".into()),
                timestamp: chrono::Utc::now(),
            };
            diesel::insert_into(submission_history::table)
                .values(&history)
                .execute(connection)?;

            let mut query = diesel::delete(submissions::table)
                .filter(submissions::id.eq(submission_id))
                .into_boxed();

            if !authenticated.has_permission(db, Permission::SubmissionReview)? {
                query = query
                    .filter(submissions::submitted_by.eq(authenticated.user_id))
                    .filter(submissions::status.eq(SubmissionStatus::Pending));
            }

            query.execute(connection)?;
            Ok(())
        })?;
        Ok(())
    }
}
