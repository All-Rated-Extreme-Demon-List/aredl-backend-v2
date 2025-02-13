use std::sync::Arc;
use actix_web::web;
use actix_web::{HttpResponse, Result, get, post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::{OpenApi, ToSchema};
use crate::auth::Authenticated;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::auth::{Permission, UserAuth};
use crate::users::merge::requests::{MergeRequestPage, MergeRequest, MergeRequestUpsert};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct MergeRequestOptions {
	/// The secondary user to merge, whose data will be merged into the authenticated user.
	pub secondary_user: Uuid,
}

#[utoipa::path(
    get,
    summary = "[Staff]Get merge requests",
    description = "Paginated list of pending/denied merge requests",
    tag = "Users - Merges",
    params(
		("page" = Option<i64>, Query, description = "The page of the merge requests to fetch"),
		("per_page" = Option<i64>, Query, description = "The number of merge requests to fetch per page"),
	),
    responses(
        (status = 200, body = Paginated<MergeRequest>)
    ),
    security(
        ("access_token" = ["MergeReview"]),
        ("api_key" = ["MergeReview"]),
    )
)]
#[get("", wrap="UserAuth::require(Permission::MergeReview)")]
async fn list(db: web::Data<Arc<DbAppState>>, page_query: web::Query<PageQuery<20>>) -> Result<HttpResponse, ApiError> {
	let result = web::block(move || {
        let mut conn = db.connection()?;
        MergeRequestPage::find_all(&mut conn, page_query.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Auth]Create merge request",
    description = "Creates a new merge request for the given user (secondary user) to be merged into the authenticated user (primary user).",
    tag = "Users - Merges",
    request_body = MergeRequestOptions,
    responses(
        (status = 200, body = MergeRequest)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[post("", wrap="UserAuth::load()")]
async fn create(db: web::Data<Arc<DbAppState>>, options: web::Json<MergeRequestOptions>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
	let result = web::block(move || {
        let mut conn = db.connection()?;
		let merge_upsert = MergeRequestUpsert { primary_user: authenticated.user_id, secondary_user: options.secondary_user };
        MergeRequest::upsert(&mut conn, merge_upsert)
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Accept merge request",
    description = "Accepts an existing merge request, merges both users and deletes the request.",
    tag = "Users - Merges",
    responses(
        (status = 200, body = MergeRequest)
    ),
    security(
        ("access_token" = ["MergeReview"]),
        ("api_key" = ["MergeReview"]),
    )
)]
#[post("/{id}/accept", wrap="UserAuth::require(Permission::MergeReview)")]
async fn accept(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
	let result = web::block(move || {
        let mut conn = db.connection()?;
        MergeRequest::accept(&mut conn, id.into_inner())
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Deny merge request",
    description = "Rejects an existing merge request. Does not delete the request, but marks it as rejected.",
    tag = "Users - Merges",
    responses(
        (status = 200, body = MergeRequest)
    ),
    security(
        ("access_token" = ["MergeReview"]),
        ("api_key" = ["MergeReview"]),
    )
)]
#[post("/{id}/reject", wrap="UserAuth::require(Permission::MergeReview)")]
async fn reject(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
	let result = web::block(move || {
        let mut conn = db.connection()?;
        MergeRequest::reject(&mut conn, id.into_inner())
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            MergeRequest,
			MergeRequestPage,
			MergeRequestOptions
        )
    ),
    paths(
		list,
		create,
        accept,
        reject
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/requests")
            .service(list)
            .service(create)
            .service(accept)
            .service(reject)
    );
}