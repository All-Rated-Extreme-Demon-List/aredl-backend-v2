use crate::{
    aredl::submissions::{Submission, SubmissionStatus},
    custom_schema::aredl_submissions_with_priority,
    db::DbAppState,
    error_handler::ApiError,
    schema::aredl_submissions,
};
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionQueue {
    /// The amount of pending submissions in the database.
    pub levels_in_queue: i32,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct QueuePositionResponse {
    pub position: i64,
    pub total: i64,
}

impl Submission {
    pub fn get_queue_position(
        db: web::Data<Arc<DbAppState>>,
        submission_id: Uuid,
    ) -> Result<(i64, i64), ApiError> {
        let conn = &mut db.connection()?;

        // Get the priority and created_at of the target submission
        let (target_priority, target_created_at): (i64, NaiveDateTime) =
            aredl_submissions_with_priority::table
                .filter(aredl_submissions_with_priority::id.eq(submission_id))
                .filter(aredl_submissions_with_priority::status.eq(SubmissionStatus::Pending))
                .select((
                    aredl_submissions_with_priority::priority_value,
                    aredl_submissions_with_priority::created_at,
                ))
                .first(conn)?;

        // Count how many pending submissions come before this one
        let position = aredl_submissions_with_priority::table
            .filter(aredl_submissions_with_priority::status.eq(SubmissionStatus::Pending))
            .filter(
                aredl_submissions_with_priority::priority_value
                    .gt(target_priority)
                    .or(aredl_submissions_with_priority::priority_value
                        .eq(target_priority)
                        .and(aredl_submissions_with_priority::created_at.lt(target_created_at))),
            )
            .count()
            .get_result::<i64>(conn)?
            + 1;

        // Total number of pending submissions
        let total = aredl_submissions_with_priority::table
            .filter(aredl_submissions_with_priority::status.eq(SubmissionStatus::Pending))
            .count()
            .get_result::<i64>(conn)?;

        Ok((position, total))
    }
}

impl SubmissionQueue {
    pub fn get_queue(db: web::Data<Arc<DbAppState>>) -> Result<Self, ApiError> {
        let conn = &mut db.connection()?;

        let levels = aredl_submissions::table
            .filter(aredl_submissions::status.eq(SubmissionStatus::Pending))
            .count()
            .get_result::<i64>(conn)? as i32;

        Ok(Self {
            levels_in_queue: levels,
        })
    }
}
