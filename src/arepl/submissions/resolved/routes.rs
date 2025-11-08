use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    arepl::submissions::{
        resolved::{ResolvedSubmissionPage, SubmissionQueryOptions},
        SubmissionPage, SubmissionResolved,
    },
    auth::{Authenticated, Permission, UserAuth},
    db::DbAppState,
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
};

#[utoipa::path(
    get,
    summary = "[Staff]List submissions",
    description = "Get a possibly filtered list of resolved submissions.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = Paginated<ResolvedSubmissionPage>)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
)]
#[get("", wrap = "UserAuth::require(Permission::SubmissionReview)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<50>>,
    options: web::Query<SubmissionQueryOptions>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(move || {
        ResolvedSubmissionPage::find_all(
            &mut db.connection()?,
            page_query.into_inner(),
            options.into_inner(),
            authenticated,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(submissions))
}

#[utoipa::path(
    get,
    summary = "[Auth]Get a resolved submission",
    description = "Get a specific submission by its ID. If you aren't staff, the submission must be yours.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = SubmissionResolved)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
)]
#[get("{id}", wrap = "UserAuth::load()")]
async fn find_one(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let submission = web::block(move || {
        SubmissionResolved::find_one(&mut db.connection()?, id.into_inner(), authenticated)
    })
    .await??;
    Ok(HttpResponse::Ok().json(submission))
}

#[utoipa::path(
    get,
    summary = "[Auth]Get own submissions",
    description = "List all submissions submitted by the logged in user.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = Paginated<SubmissionPage>)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[get("@me", wrap = "UserAuth::load()")]
async fn me(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<50>>,
    options: web::Query<SubmissionQueryOptions>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(move || {
        ResolvedSubmissionPage::find_own(
            &mut db.connection()?,
            page_query.into_inner(),
            options.into_inner(),
            authenticated,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(submissions))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        SubmissionPage,
        SubmissionQueryOptions,
        SubmissionResolved,
        ResolvedSubmissionPage,
    )),
    paths(find_all, find_one, me,)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(find_all).service(me).service(find_one);
}
