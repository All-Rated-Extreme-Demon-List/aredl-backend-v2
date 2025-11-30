use crate::{
    app_data::db::DbConnection,
    aredl::levels::ExtendedBaseLevel,
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    schema::aredl::{submissions, submissions_with_priority},
    users::BaseUser,
};
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, Connection, ExpressionMethods, QueryDsl, RunQueryDsl, Selectable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq, Default)]
#[ExistingTypePath = "crate::schema::aredl::sql_types::SubmissionStatus"]
#[DbValueStyle = "PascalCase"]
pub enum SubmissionStatus {
    #[default]
    Pending,
    Claimed,
    UnderConsideration,
    Denied,
    Accepted,
    UnderReview,
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
    /// Private notes given by the reviewer when reviewing the record.
    pub private_reviewer_notes: Option<String>,
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
    /// Private notes given by the reviewer when reviewing the record.
    pub private_reviewer_notes: Option<String>,
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
    /// [MOD ONLY] User who reviewed the record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<BaseUser>,
    /// [MOD ONLY] Private notes given by the reviewer when reviewing the record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_reviewer_notes: Option<String>,
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
    pub priority_value: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionPage {
    data: Vec<Submission>,
}

impl Submission {
    pub fn claim_highest_priority(
        conn: &mut DbConnection,
        authenticated: Authenticated,
    ) -> Result<SubmissionResolved, ApiError> {
        conn.transaction(|conn| -> Result<SubmissionResolved, ApiError> {
            let next_id: Uuid = submissions_with_priority::table
                .filter(submissions_with_priority::status.eq(SubmissionStatus::Pending))
                // prevent moderators from claiming their own submissions
                .filter(submissions_with_priority::submitted_by.ne(authenticated.user_id))
                .for_update()
                .skip_locked()
                .order((
                    submissions_with_priority::priority.desc(),
                    submissions_with_priority::priority_value.desc(),
                    submissions_with_priority::created_at.asc(),
                ))
                .select(submissions_with_priority::id)
                .first(conn)?;

            diesel::update(submissions::table.filter(submissions::id.eq(next_id)))
                .set((
                    submissions::status.eq(SubmissionStatus::Claimed),
                    submissions::reviewer_id.eq(authenticated.user_id),
                    submissions::updated_at.eq(chrono::Utc::now()),
                ))
                .execute(conn)?;

            let resolved = SubmissionResolved::find_one(conn, next_id, authenticated)?;

            Ok(resolved)
        })
    }

    pub fn delete(
        conn: &mut DbConnection,
        submission_id: Uuid,
        authenticated: Authenticated,
    ) -> Result<(), ApiError> {
        conn.transaction(|connection| -> Result<(), ApiError> {
            let mut query = diesel::delete(submissions::table)
                .filter(submissions::id.eq(submission_id))
                .into_boxed();

            if !authenticated.has_permission(connection, Permission::SubmissionReview)? {
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
