use crate::{
    aredl::levels::{BaseLevel, ResolvedLevel},
    custom_schema::aredl_submissions_with_priority,
    db::DbAppState,
    error_handler::ApiError,
    schema::{
        aredl_submissions, submission_history, users,
    },
    users::BaseUser,
};
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::{
    pg::Pg,
    ExpressionMethods, QueryDsl, RunQueryDsl, Selectable,
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::SubmissionStatus"]
#[DbValueStyle = "PascalCase"]
pub enum SubmissionStatus {
    Pending,
    Claimed,
    UnderConsideration,
    Denied,
    // Accepted (unused)
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
#[diesel(table_name = aredl_submissions, check_for_backend(Pg))]
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
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Selectable, Debug, ToSchema)]
#[diesel(table_name = aredl_submissions_with_priority, check_for_backend(Pg))]
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
    pub created_at: NaiveDateTime,
    /// The priority value of this submission
    pub priority_value: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionResolved {
    /// Internal UUID of the submission.
    pub id: Uuid,
    /// The level this submission is on
    pub level: BaseLevel,
    /// Internal UUID of the submitter.
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
    /// Internal UUID of the user who reviewed the record.
    pub reviewer: Option<BaseUser>,
    /// Whether the record was submitted as a priority record.
    pub priority: bool,
    /// Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
    /// Timestamp of when the submission was created.
    pub created_at: NaiveDateTime,
    ///
    pub priority_value: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ReviewerNotes {
    pub notes: Option<String>,
}

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

impl SubmissionResolved {
    pub fn from(
        submission: Submission,
        db: web::Data<Arc<DbAppState>>,
        priority: Option<i64>,
    ) -> Result<SubmissionResolved, ApiError> {
        let conn = &mut db.connection()?;
        let level = ResolvedLevel::find(db, submission.level_id)?;
        let base_level = BaseLevel {
            id: level.id,
            name: level.name,
        };

        let submitter = users::table
            .filter(users::id.eq(submission.submitted_by))
            .select((users::username, users::global_name))
            .first::<(String, String)>(conn)?;
        let submitted_by = BaseUser {
            id: submission.submitted_by,
            username: submitter.0,
            global_name: submitter.1,
        };

        let reviewer: Option<BaseUser> = match submission.reviewer_id {
            Some(reviewer_id) => {
                let reviewer_db = users::table
                    .filter(users::id.eq(reviewer_id))
                    .select((users::username, users::global_name))
                    .first::<(String, String)>(conn)?;
                Some(BaseUser {
                    id: reviewer_id,
                    username: reviewer_db.0,
                    global_name: reviewer_db.1,
                })
            }
            None => None,
        };

        let priority_value = match priority {
            None => aredl_submissions_with_priority::table
                .filter(aredl_submissions_with_priority::id.eq(submission.id))
                .select(aredl_submissions_with_priority::priority_value)
                .first::<i64>(conn)?,
            Some(v) => v,
        };
        Ok(SubmissionResolved {
            id: submission.id,
            level: base_level,
            submitted_by,
            mobile: submission.mobile,
            ldm_id: submission.ldm_id,
            video_url: submission.video_url,
            raw_url: submission.raw_url,
            mod_menu: submission.mod_menu,
            status: submission.status,
            reviewer,
            priority: submission.priority,
            reviewer_notes: submission.reviewer_notes,
            user_notes: submission.user_notes,
            created_at: submission.created_at,
            priority_value,
        })
    }
}
