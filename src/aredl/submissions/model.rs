use crate::{
    app_data::db::DbConnection,
    aredl::levels::ExtendedBaseLevel,
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    schema::aredl::submissions,
    users::ExtendedBaseUser,
};
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, sql_types::Bool, BoxableExpression, Selectable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
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

impl SubmissionStatus {
    pub fn websocket_type(&self) -> Option<&'static str> {
        match self {
            SubmissionStatus::Accepted => Some("SUBMISSION_ACCEPTED"),
            SubmissionStatus::Denied => Some("SUBMISSION_DENIED"),
            SubmissionStatus::UnderConsideration => Some("SUBMISSION_UNDER_CONSIDERATION"),
            SubmissionStatus::UnderReview => Some("SUBMISSION_UNDER_REVIEW"),
            SubmissionStatus::Claimed | SubmissionStatus::Pending => None,
        }
    }
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
    /// ID of the custom copy used for the record, if any.
    pub custom_copy_id: Option<i32>,
    /// Completion video URL.
    ///
    /// The provider is enforced and the URL is stored in a standardized canonical form.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub video_url: String,
    /// Raw footage URL (optional).
    ///
    /// Only requires a valid URL (the site is not enforced). If the URL matches a recognized provider
    /// it is standardized, otherwise it is stored as-is.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// The status of this submission
    pub status: SubmissionStatus,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Whether the record was submitted as a priority record.
    pub priority: bool,
    /// Timestamp used to order priority submissions in the prio queue.
    pub priority_at: DateTime<Utc>,
    /// Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
    /// Private notes given by the reviewer when reviewing the record.
    pub private_reviewer_notes: Option<String>,
    /// Whether or not this submission has been locked by a staff member
    pub locked: bool,
    /// Timestamp of when the submission was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the submission was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionResolved {
    /// Internal UUID of the submission.
    pub id: Uuid,
    /// The level this submission is for
    pub level: ExtendedBaseLevel,
    /// User who submitted this completion.
    pub submitted_by: ExtendedBaseUser,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// ID of the custom copy used for the record, if any.
    pub custom_copy_id: Option<i32>,
    /// Completion video URL.
    ///
    /// The provider is enforced and the URL is stored in a standardized canonical form.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub video_url: String,
    /// Raw footage URL (optional).
    ///
    /// Only requires a valid URL (the site is not enforced). If the URL matches a recognized provider
    /// it is standardized, otherwise it is stored as-is.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// The status of this submission
    pub status: SubmissionStatus,
    /// [MOD ONLY] User who reviewed the record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<ExtendedBaseUser>,
    /// [MOD ONLY] Private notes given by the reviewer when reviewing the record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_reviewer_notes: Option<String>,
    /// Whether the record was submitted as a priority record.
    pub priority: bool,
    /// Timestamp used to order priority submissions in the prio queue.
    pub priority_at: DateTime<Utc>,
    /// Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
    /// Whether or not this submission has been locked by a staff member
    pub locked: bool,
    /// Timestamp of when the submission was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the submission was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionPage {
    data: Vec<Submission>,
}

pub type SubmissionFilter = Box<dyn BoxableExpression<submissions::table, Pg, SqlType = Bool>>;

impl Submission {
    // filters for submissions that can be claimed by the current reviewer
    fn claimable_filter(
        reviewer_id: Uuid,
        can_claim_raw_footage: bool,
        priority: bool,
    ) -> SubmissionFilter {
        let base = submissions::submitted_by
            // prevent reviewers from claiming their own submissions
            .ne(reviewer_id)
            .and(submissions::priority.eq(priority));

        if can_claim_raw_footage {
            Box::new(
                base.and(
                    submissions::status
                        .eq(SubmissionStatus::Pending)
                        .or(submissions::status.eq(SubmissionStatus::UnderReview)),
                ),
            )
        } else {
            Box::new(
                base.and(submissions::status.eq(SubmissionStatus::Pending))
                    .and(submissions::raw_url.is_null()),
            )
        }
    }

    // Picks the next submission (if any) in the queue/prio queue, depending on the reviewer's permissions
    fn find_next_claimable_id(
        conn: &mut DbConnection,
        reviewer_id: Uuid,
        can_claim_raw_footage: bool,
        priority: bool,
    ) -> Result<Option<Uuid>, ApiError> {
        let query = submissions::table
            .filter(Self::claimable_filter(
                reviewer_id,
                can_claim_raw_footage,
                priority,
            ))
            .for_update()
            .skip_locked()
            .select(submissions::id);

        // for the priority queue, order by last update, so that people who just got aredl+ are placed at the end of the prio queue
        // (this however makes any update/resubmit have to go through the prio queue again, which is fine since it is barely anything
        // compared to the main queue, for which we prefer using created_at instead)

        let next_id = if priority {
            query
                .order(submissions::priority_at.asc())
                .first(conn)
                .optional()?
        } else {
            query
                .order(submissions::created_at.asc())
                .first(conn)
                .optional()?
        };

        Ok(next_id)
    }

    // Picks and claims the next submission in the prio queue, or if empty, the main queue.
    pub fn claim_highest_priority(
        conn: &mut DbConnection,
        authenticated: &Authenticated,
    ) -> Result<SubmissionResolved, ApiError> {
        conn.transaction(|conn| -> Result<SubmissionResolved, ApiError> {
            let can_claim_raw_footage =
                authenticated.has_permission(conn, Permission::SubmissionEditWithRawFootage)?;

            let preferred_id = Self::find_next_claimable_id(
                conn,
                authenticated.user_id,
                can_claim_raw_footage,
                true,
            )?;

            let next_id = if let Some(id) = preferred_id {
                id
            } else if let Some(id) = Self::find_next_claimable_id(
                conn,
                authenticated.user_id,
                can_claim_raw_footage,
                false,
            )? {
                id
            } else {
                return Err(ApiError::NotFound(
                    "There are no submissions available to claim",
                ));
            };

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
        authenticated: &Authenticated,
    ) -> Result<(), ApiError> {
        conn.transaction(|connection| -> Result<(), ApiError> {
            let mut query = diesel::delete(submissions::table)
                .filter(submissions::id.eq(submission_id))
                .into_boxed();

            if !authenticated
                .has_permission(connection, Permission::SubmissionEditNonSelfClaimed)?
            {
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
