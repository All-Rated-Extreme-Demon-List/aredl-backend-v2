use crate::{
    aredl::{levels::ExtendedBaseLevel, submissions::*},
    auth::{Authenticated, Permission},
    db::{DbAppState, DbConnection},
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
    schema::{
        aredl::{levels, submission_history, submissions_with_priority},
        users,
    },
    users::BaseUser,
};
use actix_web::web;
use diesel::{expression_methods::NullableExpressionMethods, BoolExpressionMethods, PgTextExpressionMethods};
use diesel::{
    pg::Pg, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, Selectable, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

pub type ResolvedSubmissionRow = (
    SubmissionWithPriority,
    ExtendedBaseLevel,
    BaseUser,
    Option<BaseUser>,
);

fn get_submission_ids_from_notes(note_text: String, conn: &mut DbConnection) -> Result<Vec<Uuid>, ApiError> {
    Ok(submission_history::table
        .filter(
            submission_history::user_notes.ilike(&note_text)
                .or(submission_history::reviewer_notes.ilike(&note_text)),
        )
        .select(submission_history::submission_id)
        .distinct()
        .load(conn)?)
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionQueryOptions {
    pub status_filter: Option<SubmissionStatus>,
    pub mobile_filter: Option<bool>,
    pub level_filter: Option<Uuid>,
    pub submitter_filter: Option<Uuid>,
    pub priority_filter: Option<bool>,
    pub reviewer_filter: Option<Uuid>,
    pub note_filter: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResolvedSubmissionPage {
    data: Vec<SubmissionResolved>,
}

#[macro_export]
macro_rules! aredl_base_resolved_submission_query {
    () => {{
        let reviewers = alias!(users as reviewers);

        submissions_with_priority::table
            .inner_join(levels::table.on(submissions_with_priority::level_id.eq(levels::id)))
            .inner_join(users::table.on(users::id.eq(submissions_with_priority::submitted_by)))
            .left_join(
                reviewers.on(reviewers
                    .field(users::id)
                    .nullable()
                    .eq(submissions_with_priority::reviewer_id.nullable())),
            )
            .select((
                SubmissionWithPriority::as_select(),
                ExtendedBaseLevel::as_select(),
                BaseUser::as_select(),
                reviewers
                    .fields(<BaseUser as Selectable<Pg>>::construct_selection())
                    .nullable(),
            ))
            .into_boxed::<Pg>()
    }};
}

#[macro_export]
macro_rules! aredl_apply_submissions_filters {
    ($query:expr, $opts:expr, $conn:expr) => {{
        let opts = &$opts;
        let mut new_query = $query;
        let conn = $conn;

        if let Some(status) = opts.status_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::status.eq(status));
        }
        if let Some(mobile) = opts.mobile_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::mobile.eq(mobile));
        }
        if let Some(level) = opts.level_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::level_id.eq(level));
        }
        if let Some(submitter) = opts.submitter_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::submitted_by.eq(submitter));
        }
        if let Some(priority) = opts.priority_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::priority.eq(priority));
        }
        if let Some(reviewer) = opts.reviewer_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::reviewer_id.eq(reviewer));
        }

        if let Some(note_text) = opts.note_filter.clone() {
            let ids = get_submission_ids_from_notes(note_text, conn)?;
            new_query = new_query.filter(submissions_with_priority::id.eq_any(ids));
        }

        new_query
    }};
}

#[macro_export]
macro_rules! aredl_apply_submissions_filters_unauth {
    ($query:expr, $opts:expr) => {{
        let opts = &$opts;
        let mut new_query = $query;

        if let Some(status) = opts.status_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::status.eq(status));
        }
        if let Some(mobile) = opts.mobile_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::mobile.eq(mobile));
        }
        if let Some(level) = opts.level_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::level_id.eq(level));
        }
        if let Some(submitter) = opts.submitter_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::submitted_by.eq(submitter));
        }
        if let Some(priority) = opts.priority_filter.clone() {
            new_query = new_query.filter(submissions_with_priority::priority.eq(priority));
        }

        new_query
    }};
}

impl SubmissionResolved {
    pub fn from_data(resolved: ResolvedSubmissionRow) -> SubmissionResolved {
        let (submission, level, submitter, reviewer) = resolved;
        SubmissionResolved {
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
            updated_at: submission.updated_at,
            priority_value: submission.priority_value,
        }
    }

    pub fn resolve_from_id(
        submission_id: Uuid,
        db: web::Data<Arc<DbAppState>>,
        authenticated: Authenticated,
    ) -> Result<SubmissionResolved, ApiError> {
        let conn = &mut db.connection()?;

        let resolved_raw = aredl_base_resolved_submission_query!()
            .filter(submissions_with_priority::id.eq(submission_id))
            .first::<ResolvedSubmissionRow>(conn)?;

        let mut resolved = Self::from_data(resolved_raw);

        if !authenticated.has_permission(db.clone(), Permission::SubmissionReview)? {
            resolved.reviewer = None;
        }

        Ok(resolved)
    }

    pub fn find_one(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
        authenticated: Authenticated,
    ) -> Result<SubmissionResolved, ApiError> {
        let conn = &mut db.connection()?;

        let mut query =
            aredl_base_resolved_submission_query!().filter(submissions_with_priority::id.eq(id));

        if !authenticated.has_permission(db.clone(), Permission::SubmissionReview)? {
            query = query.filter(submissions_with_priority::submitted_by.eq(authenticated.user_id));
        }

        let resolved = query.first::<ResolvedSubmissionRow>(conn)?;

        Ok(Self::from_data(resolved))
    }
}

impl ResolvedSubmissionPage {
    pub fn find_all<const D: i64>(
        db: web::Data<Arc<DbAppState>>,
        page_query: PageQuery<D>,
        options: SubmissionQueryOptions,
        authenticated: Authenticated,
    ) -> Result<Paginated<Self>, ApiError> {
        let conn = &mut db.connection()?;

        let mut query = aredl_base_resolved_submission_query!();
        query = aredl_apply_submissions_filters!(query, options, &mut db.connection()?);

        let submissions = query
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .order(submissions_with_priority::created_at.desc())
            .load::<ResolvedSubmissionRow>(conn)?;

        let mut submissions = submissions
            .into_iter()
            .map(|resolved_row| SubmissionResolved::from_data(resolved_row))
            .collect::<Vec<_>>();

        let mut count_query = aredl_base_resolved_submission_query!();
        count_query = aredl_apply_submissions_filters!(count_query, options, &mut db.connection()?);
        let total_count: i64 = count_query.count().get_result(conn)?;

        if !authenticated.has_permission(db, Permission::SubmissionReview)? {
            submissions.iter_mut().for_each(|s| s.reviewer = None);
        }

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
        let options = SubmissionQueryOptions {
            submitter_filter: Some(authenticated.user_id),
            ..options
        };

        Ok(Self::find_all(db, page_query, options, authenticated)?)
    }
}
