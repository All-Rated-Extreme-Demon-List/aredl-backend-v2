use crate::{
    app_data::db::DbConnection,
    aredl::submissions::{Submission, SubmissionFilter, SubmissionStatus},
    error_handler::ApiError,
    schema::aredl::submissions,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionQueue {
    /// The amount of pending submissions that are not marked as priority.
    pub regular_submissions_in_queue: i64,
    /// The amount of pending submissions that are not marked as priority and do not have raw footage.
    pub regular_submissions_without_raw_in_queue: i64,
    /// The amount of pending submissions that are marked as priority.
    pub priority_submissions_in_queue: i64,
    /// The amount of pending submissions that are marked as priority and do not have raw footage.
    pub priority_submissions_without_raw_in_queue: i64,
    /// The amount of submissions currently under consideration.
    pub uc_submissions: i64,
    /// The timestamp of the oldest pending submission in the queue, if any.
    pub oldest_submission: Option<DateTime<Utc>>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct QueuePositionResponse {
    /// The position of the submission in its queue (regular or priority).
    pub position: i64,
    /// Whether the submission is in the priority queue or not.
    pub priority: bool,
}

impl Submission {
    // filters out all submissions that come after the target one
    fn queue_position_filter(
        target_priority: bool,
        target_created_at: DateTime<Utc>,
        target_priority_at: DateTime<Utc>,
        target_has_raw_footage: bool,
    ) -> SubmissionFilter {
        let mut base_filter: SubmissionFilter =
            Box::new(submissions::status.eq(SubmissionStatus::Pending));

        if !target_has_raw_footage {
            base_filter = Box::new(base_filter.and(submissions::raw_url.is_null()));
        }

        if target_priority {
            Box::new(
                base_filter
                    .and(submissions::priority.eq(true))
                    .and(submissions::priority_at.lt(target_priority_at)),
            )
        } else {
            Box::new(
                base_filter
                    .and(submissions::priority.eq(false))
                    .and(submissions::created_at.lt(target_created_at)),
            )
        }
    }

    pub fn get_queue_position(
        conn: &mut DbConnection,
        submission_id: Uuid,
    ) -> Result<(i64, bool), ApiError> {
        let (target_priority, target_created_at, target_priority_at, target_has_raw_footage): (
            bool,
            DateTime<Utc>,
            DateTime<Utc>,
            bool,
        ) = submissions::table
            .filter(submissions::id.eq(submission_id))
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            .select((
                submissions::priority,
                submissions::created_at,
                submissions::priority_at,
                submissions::raw_url.is_not_null(),
            ))
            .first(conn)?;

        // Count how many pending submissions come before this one
        let position = submissions::table
            .filter(Self::queue_position_filter(
                target_priority,
                target_created_at,
                target_priority_at,
                target_has_raw_footage,
            ))
            .count()
            .get_result::<i64>(conn)?
            + 1;

        Ok((position, target_priority))
    }
}

impl SubmissionQueue {
    fn pending_filter(priority: bool, exclude_raw: bool) -> SubmissionFilter {
        let base_pending_filter: SubmissionFilter = Box::new(
            submissions::status
                .eq(SubmissionStatus::Pending)
                .and(submissions::priority.eq(priority)),
        );

        if exclude_raw {
            Box::new(base_pending_filter.and(submissions::raw_url.is_null()))
        } else {
            base_pending_filter
        }
    }

    pub fn get_queue(conn: &mut DbConnection) -> Result<Self, ApiError> {
        let regular_submissions_in_queue = submissions::table
            .filter(Self::pending_filter(false, false))
            .count()
            .get_result::<i64>(conn)?;

        let regular_submissions_without_raw_in_queue = submissions::table
            .filter(Self::pending_filter(false, true))
            .count()
            .get_result::<i64>(conn)?;

        let priority_submissions_in_queue = submissions::table
            .filter(Self::pending_filter(true, false))
            .count()
            .get_result::<i64>(conn)?;

        let priority_submissions_without_raw_in_queue = submissions::table
            .filter(Self::pending_filter(true, true))
            .count()
            .get_result::<i64>(conn)?;

        let uc_submissions = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::UnderConsideration))
            .count()
            .get_result::<i64>(conn)?;

        let oldest_submission = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            .select(submissions::updated_at)
            .order(submissions::updated_at.asc())
            .first::<DateTime<Utc>>(conn)
            .optional()?;

        Ok(Self {
            regular_submissions_in_queue,
            regular_submissions_without_raw_in_queue,
            priority_submissions_in_queue,
            priority_submissions_without_raw_in_queue,
            uc_submissions,
            oldest_submission,
        })
    }
}
