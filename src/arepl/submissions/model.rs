use crate::{
    app_data::db::DbConnection,
    arepl::levels::ExtendedBaseLevel,
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    schema::arepl::submissions,
    users::ExtendedBaseUser,
};
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg, Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, Selectable,
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
use utoipa::ToSchema;
use uuid::Uuid;

static CLAIM_PREFER_PRIORITY: OnceLock<Mutex<HashMap<Uuid, bool>>> = OnceLock::new();

fn prefer_priority_next(reviewer_id: Uuid) -> bool {
    let lock = CLAIM_PREFER_PRIORITY.get_or_init(|| Mutex::new(HashMap::new()));
    let mut state = lock.lock().expect("claim preference lock poisoned");
    *state.entry(reviewer_id).or_insert(true)
}

fn set_prefer_priority_next(reviewer_id: Uuid, prefer_priority: bool) {
    let lock = CLAIM_PREFER_PRIORITY.get_or_init(|| Mutex::new(HashMap::new()));
    let mut state = lock.lock().expect("claim preference lock poisoned");
    state.insert(reviewer_id, prefer_priority);
}

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq, Default)]
#[ExistingTypePath = "crate::schema::arepl::sql_types::SubmissionStatus"]
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
    /// Completion video URL.
    ///
    /// The provider is enforced and the URL is stored in a standardized canonical form.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub video_url: String,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
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
    /// Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
    /// Private notes given by the reviewer when reviewing the record.
    pub private_reviewer_notes: Option<String>,
    /// Whether or not this submission has been locked by a staff member
    pub locked: bool,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
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
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Completion video URL.
    ///
    /// The provider is enforced and the URL is stored in a standardized canonical form.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub video_url: String,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
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
    /// Whether the record was submitted as a priority record.
    pub priority: bool,
    /// Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
    /// Whether or not this submission has been locked by a staff member
    pub locked: bool,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
    /// [MOD ONLY] Private notes given by the reviewer when reviewing the record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_reviewer_notes: Option<String>,
    /// Timestamp of when the submission was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the submission was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionPage {
    data: Vec<Submission>,
}

impl Submission {
    fn find_next_claimable_id(
        conn: &mut DbConnection,
        reviewer_id: Uuid,
        priority: bool,
    ) -> Result<Option<Uuid>, ApiError> {
        let next_id = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            // prevent moderators from claiming their own submissions
            .filter(submissions::submitted_by.ne(reviewer_id))
            .filter(submissions::priority.eq(priority))
            .for_update()
            .skip_locked()
            .order(submissions::created_at.asc())
            .select(submissions::id)
            .first::<Uuid>(conn)
            .optional()?;

        Ok(next_id)
    }

    pub fn claim_highest_priority(
        conn: &mut DbConnection,
        authenticated: Authenticated,
    ) -> Result<SubmissionResolved, ApiError> {
        conn.transaction(|conn| -> Result<SubmissionResolved, ApiError> {
            let prefer_priority = prefer_priority_next(authenticated.user_id);

            let preferred_id =
                Self::find_next_claimable_id(conn, authenticated.user_id, prefer_priority)?;

            let (next_id, claimed_priority) = if let Some(id) = preferred_id {
                (id, prefer_priority)
            } else if let Some(id) =
                Self::find_next_claimable_id(conn, authenticated.user_id, !prefer_priority)?
            {
                (id, !prefer_priority)
            } else {
                return Err(ApiError::new(
                    404,
                    "There are no submissions available to claim",
                ));
            };

            set_prefer_priority_next(authenticated.user_id, !claimed_priority);

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
