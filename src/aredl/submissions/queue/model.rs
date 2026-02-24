use crate::{
    app_data::db::DbConnection,
    aredl::submissions::{Submission, SubmissionStatus},
    error_handler::ApiError,
    schema::aredl::submissions,
};
use chrono::{DateTime, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionQueue {
    /// The amount of pending submissions that are not marked as priority.
    pub regular_submissions_in_queue: i32,
    /// The amount of pending submissions that are marked as priority.
    pub priority_submissions_in_queue: i32,
    /// The amount of submissions currently under consideration.
    pub uc_submissions: i32,
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
    pub fn get_queue_position(
        conn: &mut DbConnection,
        submission_id: Uuid,
    ) -> Result<(i64, bool), ApiError> {
        // Get the priority and created_at of the target submission
        let (target_priority, target_created_at): (bool, DateTime<Utc>) = submissions::table
            .filter(submissions::id.eq(submission_id))
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            .select((submissions::priority, submissions::created_at))
            .first(conn)?;

        // Count how many pending submissions come before this one
        let position = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            .filter(
                submissions::priority
                    .eq(target_priority)
                    .and(submissions::created_at.lt(target_created_at)),
            )
            .count()
            .get_result::<i64>(conn)?
            + 1;

        Ok((position, target_priority))
    }
}

impl SubmissionQueue {
    pub fn get_queue(conn: &mut DbConnection) -> Result<Self, ApiError> {
        let regular_submissions_in_queue = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            .filter(submissions::priority.eq(false))
            .count()
            .get_result::<i64>(conn)? as i32;

        let priority_submissions_in_queue = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            .filter(submissions::priority.eq(true))
            .count()
            .get_result::<i64>(conn)? as i32;

        let uc_submissions = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::UnderConsideration))
            .count()
            .get_result::<i64>(conn)? as i32;

        let oldest_submission = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            .select(submissions::updated_at)
            .order(submissions::updated_at.asc())
            .first::<DateTime<Utc>>(conn)
            .optional()?;

        Ok(Self {
            regular_submissions_in_queue,
            priority_submissions_in_queue,
            uc_submissions,
            oldest_submission,
        })
    }
}
