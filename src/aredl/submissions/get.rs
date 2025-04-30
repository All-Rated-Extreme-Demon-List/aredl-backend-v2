use crate::{
    aredl::levels::BaseLevel,
    aredl::submissions::*,
    auth::{Authenticated, Permission},
    custom_schema::aredl_submissions_with_priority,
    db::DbAppState,
    error_handler::ApiError,
    page_helper::PageQuery,
    page_helper::Paginated,
    schema::{
        aredl_levels,aredl_submissions, submission_history, 
        users,
    },
    users::BaseUser,
};
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::expression_methods::{BoolExpressionMethods, NullableExpressionMethods};
use diesel::{
    pg::Pg,
    sql_types::Bool,
    BoxableExpression, ExpressionMethods, IntoSql, JoinOnDsl, QueryDsl, RunQueryDsl, 
    Selectable, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResolvedSubmissionPage {
    data: Vec<SubmissionResolved>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionQueue {
    /// The amount of pending submissions in the database.
    pub levels_in_queue: i32,
}

impl SubmissionResolved {
    pub fn find_one(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
        authenticated: Authenticated,
    ) -> Result<SubmissionResolved, ApiError> {
        let conn = &mut db.connection()?;
        let has_auth = Authenticated::has_permission(
            &authenticated,
            db.clone(),
            Permission::SubmissionReview,
        )?;

        let mut query = aredl_submissions::table
            .filter(aredl_submissions::id.eq(id))
            .into_boxed();

        if !has_auth {
            query = query.filter(aredl_submissions::submitted_by.eq(authenticated.user_id));
        }

        let submission = query
            .select(Submission::as_select())
            .first(conn)?;

        let resolved = SubmissionResolved::from(submission, db, None)?;

        Ok(resolved)
    }
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

impl ResolvedSubmissionPage {
    pub fn find_all<const D: i64>(
        db: web::Data<Arc<DbAppState>>,
        page_query: PageQuery<D>,
        options: SubmissionQueryOptions,
    ) -> Result<Paginated<Self>, ApiError> {
        let conn = &mut db.connection()?;

        let reviewers = alias!(users as reviewers);

        let query = aredl_submissions_with_priority::table
            .inner_join(
                aredl_levels::table
                    .on(aredl_submissions_with_priority::level_id.eq(aredl_levels::id)),
            )
            .inner_join(
                users::table.on(users::id.eq(aredl_submissions_with_priority::submitted_by)),
            )
            .left_join(
                reviewers.on(reviewers
                    .field(users::id)
                    .nullable()
                    .eq(aredl_submissions_with_priority::reviewer_id.nullable())),
            );

        let total_count: i64 = query.count().get_result(conn)?;

        let submissions = query
            .filter(options.status_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |status| Box::new(aredl_submissions_with_priority::status.eq(status)),
            ))
            .filter(options.mobile_fiter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |mobile| Box::new(aredl_submissions_with_priority::mobile.eq(mobile)),
            ))
            .filter(options.level_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |level| Box::new(aredl_submissions_with_priority::level_id.eq(level)),
            ))
            .filter(options.submitter_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |submitter| Box::new(aredl_submissions_with_priority::submitted_by.eq(submitter)),
            ))
            .filter(options.priority_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |priority| Box::new(aredl_submissions_with_priority::priority.eq(priority)),
            ))
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((
                SubmissionWithPriority::as_select(),
                BaseLevel::as_select(),
                BaseUser::as_select(),
                reviewers
                    .fields(<BaseUser as Selectable<Pg>>::construct_selection())
                    .nullable(),
            ))
            .load::<(
                SubmissionWithPriority,
                BaseLevel,
                BaseUser,
                Option<BaseUser>,
            )>(conn)?;

        let submissions = submissions
            .into_iter()
            .map(
                |(submission, level, submitter, reviewer)| SubmissionResolved {
                    id: submission.id,
                    level,
                    submitted_by: submitter,
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
                    priority_value: submission.priority_value,
                },
            )
            .collect::<Vec<_>>();

        Ok(Paginated::<Self>::from_data(
            page_query,
            total_count,
            Self { data: submissions },
        ))
    }
    pub fn find_own<const D: i64>(
        db: web::Data<Arc<DbAppState>>,
        page_query: PageQuery<D>,
        options: SubmissionQueryOptions,
        authenticated: Authenticated,
    ) -> Result<Paginated<Self>, ApiError> {
        let conn = &mut db.connection()?;
        let reviewers = alias!(users as reviewers);
        let query = aredl_submissions_with_priority::table
            .inner_join(
                aredl_levels::table
                    .on(aredl_submissions_with_priority::level_id.eq(aredl_levels::id)),
            )
            .inner_join(
                users::table.on(users::id.eq(aredl_submissions_with_priority::submitted_by)),
            )
            .left_join(
                reviewers.on(reviewers
                    .field(users::id)
                    .nullable()
                    .eq(aredl_submissions_with_priority::reviewer_id.nullable())),
            );

        let total_count: i64 = query.count().get_result(conn)?;

        let submissions = query
            .filter(options.status_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |status| Box::new(aredl_submissions_with_priority::status.eq(status)),
            ))
            .filter(options.mobile_fiter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |mobile| Box::new(aredl_submissions_with_priority::mobile.eq(mobile)),
            ))
            .filter(options.level_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |level| Box::new(aredl_submissions_with_priority::level_id.eq(level)),
            ))
            .filter(options.priority_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |priority| Box::new(aredl_submissions_with_priority::priority.eq(priority)),
            ))
            .filter(aredl_submissions_with_priority::submitted_by.eq(authenticated.user_id))
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((
                SubmissionWithPriority::as_select(),
                BaseLevel::as_select(),
                BaseUser::as_select(),
                reviewers
                    .fields(<BaseUser as Selectable<Pg>>::construct_selection())
                    .nullable(),
            ))
            .load::<(
                SubmissionWithPriority,
                BaseLevel,
                BaseUser,
                Option<BaseUser>
            )>(conn)?;

        let submissions = submissions
            .into_iter()
            .map(
                |(submission, level, submitter, reviewer)| SubmissionResolved {
                    id: submission.id,
                    level,
                    submitted_by: submitter,
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
                    priority_value: submission.priority_value,
                },
            )
            .collect::<Vec<_>>();

        Ok(Paginated::<Self>::from_data(
            page_query,
            total_count,
            Self { data: submissions },
        ))
    }
}

impl SubmissionHistory {
    pub fn by_submission(
        db: web::Data<Arc<DbAppState>>,
        submission_id: Uuid,
    ) -> Result<Vec<SubmissionHistory>, ApiError> {
        let conn = &mut db.connection()?;

        let history = submission_history::table
            .filter(submission_history::submission_id.eq(submission_id))
            .select(SubmissionHistory::as_select())
            .order(submission_history::timestamp.desc())
            .load::<SubmissionHistory>(conn)?;

        Ok(history)
    }
}
