use crate::{
    arepl::submissions::{Submission, SubmissionStatus},
    db::DbConnection,
    error_handler::ApiError,
    schema::arepl::{submissions, submissions_with_priority},
};
use chrono::{DateTime, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionQueue {
    /// The amount of pending submissions in the database.
    pub submissions_in_queue: i32,
    /// The amount of submissions currently under consideration.
    pub uc_submissions: i32,
    /// The timestamp of the oldest pending submission in the queue, if any.
    pub oldest_submission: Option<DateTime<Utc>>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct QueuePositionResponse {
    pub position: i64,
    pub total: i64,
}

impl Submission {
    pub fn get_queue_position(
        conn: &mut DbConnection,
        submission_id: Uuid,
    ) -> Result<(i64, i64), ApiError> {
        // Get the priority and created_at of the target submission
        let (target_priority, target_created_at): (i64, DateTime<Utc>) =
            submissions_with_priority::table
                .filter(submissions_with_priority::id.eq(submission_id))
                .filter(submissions_with_priority::status.eq(SubmissionStatus::Pending))
                .select((
                    submissions_with_priority::priority_value,
                    submissions_with_priority::created_at,
                ))
                .first(conn)?;

        // Count how many pending submissions come before this one
        let position = submissions_with_priority::table
            .filter(submissions_with_priority::status.eq(SubmissionStatus::Pending))
            .filter(
                submissions_with_priority::priority_value
                    .gt(target_priority)
                    .or(submissions_with_priority::priority_value
                        .eq(target_priority)
                        .and(submissions_with_priority::created_at.lt(target_created_at))),
            )
            .count()
            .get_result::<i64>(conn)?
            + 1;

        // Total number of pending submissions
        let total = submissions_with_priority::table
            .filter(submissions_with_priority::status.eq(SubmissionStatus::Pending))
            .count()
            .get_result::<i64>(conn)?;

        Ok((position, total))
    }
}

impl SubmissionQueue {
    pub fn get_queue(conn: &mut DbConnection) -> Result<Self, ApiError> {
        let submissions_in_queue = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            .count()
            .get_result::<i64>(conn)? as i32;

        let uc_submissions = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::UnderConsideration))
            .count()
            .get_result::<i64>(conn)? as i32;

        let oldest_submission = submissions::table
            .filter(submissions::status.eq(SubmissionStatus::Pending))
            .select(submissions::created_at)
            .order(submissions::created_at.asc())
            .first::<DateTime<Utc>>(conn)
            .optional()?;

        Ok(Self {
            submissions_in_queue,
            uc_submissions,
            oldest_submission,
        })
    }
}
