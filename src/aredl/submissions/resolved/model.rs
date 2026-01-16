use crate::{
    app_data::db::DbConnection,
    aredl::{levels::ExtendedBaseLevel, submissions::*},
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
    schema::{
        aredl::{levels, submission_history, submissions_with_priority},
        users,
    },
    users::{user_filter, ExtendedBaseUser},
};
use diesel::{
    dsl::{auto_type, AliasedFields, AsSelect, Nullable},
    expression_methods::NullableExpressionMethods,
    BoolExpressionMethods, PgTextExpressionMethods,
};
use diesel::{
    pg::Pg, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, Selectable, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub type ResolvedSubmissionRow = (
    SubmissionWithPriority,
    ExtendedBaseLevel,
    ExtendedBaseUser,
    Option<ExtendedBaseUser>,
);

#[derive(Serialize, Deserialize, ToSchema)]
pub enum SubmissionsSortField {
    OldestCreatedAt,
    NewestCreatedAt,
    OldestUpdatedAt,
    NewestUpdatedAt,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubmissionQueryOptions {
    pub status_filter: Option<SubmissionStatus>,
    pub mobile_filter: Option<bool>,
    pub level_filter: Option<Uuid>,
    pub submitter_filter: Option<String>,
    pub priority_filter: Option<bool>,
    pub reviewer_filter: Option<String>,
    pub note_filter: Option<String>,
    pub sort: Option<SubmissionsSortField>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResolvedSubmissionPage {
    data: Vec<SubmissionResolved>,
}

diesel::alias!(users as reviewers: Reviewers);

#[auto_type]
fn resolve_query<'a>(q: submissions_with_priority::BoxedQuery<'a, Pg>) -> _ {
    // annoying type shenanigans to get around Diesel not being able to infer types properly
    let selection: (
        AsSelect<SubmissionWithPriority, Pg>,
        AsSelect<ExtendedBaseLevel, Pg>,
        AsSelect<ExtendedBaseUser, Pg>,
        Nullable<
            AliasedFields<
                Reviewers,
                <ExtendedBaseUser as diesel::Selectable<Pg>>::SelectExpression,
            >,
        >,
    ) = (
        SubmissionWithPriority::as_select(),
        ExtendedBaseLevel::as_select(),
        ExtendedBaseUser::as_select(),
        reviewers
            .fields(<ExtendedBaseUser as Selectable<Pg>>::construct_selection())
            .nullable(),
    );

    q.inner_join(levels::table.on(submissions_with_priority::level_id.eq(levels::id)))
        .inner_join(users::table.on(submissions_with_priority::submitted_by.eq(users::id)))
        .left_join(
            reviewers.on(reviewers
                .field(users::id)
                .nullable()
                .eq(submissions_with_priority::reviewer_id.nullable())),
        )
        .select(selection)
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
            private_reviewer_notes: submission.private_reviewer_notes,
            user_notes: submission.user_notes,
            locked: submission.locked,
            created_at: submission.created_at,
            updated_at: submission.updated_at,
            priority_value: submission.priority_value,
        }
    }

    pub fn find_one(
        conn: &mut DbConnection,
        id: Uuid,
        authenticated: Authenticated,
    ) -> Result<SubmissionResolved, ApiError> {
        let mut query = submissions_with_priority::table
            .filter(submissions_with_priority::id.eq(id))
            .into_boxed();

        let is_reviewer = authenticated.has_permission(conn, Permission::SubmissionReview)?;

        if !is_reviewer {
            query = query.filter(submissions_with_priority::submitted_by.eq(authenticated.user_id));
        }

        let mut resolved =
            Self::from_data(resolve_query(query).first::<ResolvedSubmissionRow>(conn)?);

        if !is_reviewer {
            resolved.reviewer = None;
            resolved.private_reviewer_notes = None;
        }

        Ok(resolved)
    }
}

impl ResolvedSubmissionPage {
    pub fn find_all<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: SubmissionQueryOptions,
        authenticated: Authenticated,
    ) -> Result<Paginated<Self>, ApiError> {
        let build_filtered = || {
            let mut query = submissions_with_priority::table.into_boxed::<Pg>();

            if let Some(status) = options.status_filter.clone() {
                query = query.filter(submissions_with_priority::status.eq(status));
            }

            if let Some(mobile) = options.mobile_filter.clone() {
                query = query.filter(submissions_with_priority::mobile.eq(mobile));
            }

            if let Some(level) = options.level_filter.clone() {
                query = query.filter(submissions_with_priority::level_id.eq(level));
            }

            if let Some(ref submitter) = options.submitter_filter {
                query = query.filter(
                    submissions_with_priority::submitted_by
                        .eq_any(user_filter(&submitter).select(users::id)),
                );
            }

            if let Some(priority) = options.priority_filter.clone() {
                query = query.filter(submissions_with_priority::priority.eq(priority));
            }

            if let Some(ref reviewer) = options.reviewer_filter {
                query = query.filter(
                    submissions_with_priority::reviewer_id
                        .eq_any(user_filter(&reviewer).select(users::id.nullable())),
                );
            }

            if let Some(note_text) = options.note_filter.as_deref() {
                query = query.filter(
                    submissions_with_priority::id.eq_any(
                        submission_history::table
                            .filter(
                                submission_history::user_notes
                                    .ilike(note_text)
                                    .or(submission_history::reviewer_notes.ilike(note_text)),
                            )
                            .select(submission_history::submission_id)
                            .distinct(),
                    ),
                );
            }
            query
        };

        let mut submissions_query = build_filtered();
        if let Some(sort_field) = options.sort {
            match sort_field {
                SubmissionsSortField::OldestCreatedAt => {
                    submissions_query =
                        submissions_query.order_by(submissions_with_priority::created_at.asc())
                }
                SubmissionsSortField::NewestCreatedAt => {
                    submissions_query =
                        submissions_query.order_by(submissions_with_priority::created_at.desc())
                }
                SubmissionsSortField::OldestUpdatedAt => {
                    submissions_query =
                        submissions_query.order_by(submissions_with_priority::updated_at.asc())
                }
                SubmissionsSortField::NewestUpdatedAt => {
                    submissions_query =
                        submissions_query.order_by(submissions_with_priority::updated_at.desc())
                }
            }
        } else {
            submissions_query =
                submissions_query.order_by(submissions_with_priority::created_at.desc());
        }

        let submissions: Vec<ResolvedSubmissionRow> = resolve_query(
            submissions_query
                .limit(page_query.per_page())
                .offset(page_query.offset()),
        )
        .load::<ResolvedSubmissionRow>(conn)?;

        let mut submissions = submissions
            .into_iter()
            .map(|resolved_row| SubmissionResolved::from_data(resolved_row))
            .collect::<Vec<_>>();

        let total_count: i64 = build_filtered().count().get_result(conn)?;

        if !authenticated.has_permission(conn, Permission::SubmissionReview)? {
            submissions
                .iter_mut()
                .for_each(|s: &mut SubmissionResolved| {
                    s.reviewer = None;
                    s.private_reviewer_notes = None;
                });
        }

        Ok(Paginated::<Self>::from_data(
            page_query,
            total_count,
            Self { data: submissions },
        ))
    }

    pub fn find_own<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        mut options: SubmissionQueryOptions,
        authenticated: Authenticated,
    ) -> Result<Paginated<Self>, ApiError> {
        options.submitter_filter = Some(authenticated.user_id.to_string());
        options.reviewer_filter = None;

        Ok(Self::find_all(conn, page_query, options, authenticated)?)
    }
}
