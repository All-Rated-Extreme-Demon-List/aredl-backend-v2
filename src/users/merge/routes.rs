use crate::app_data::db::DbAppState;
use crate::auth::{Permission, UserAuth};
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::users::merge::requests;
use crate::users::merge::MergeLogPage;
use crate::users::merge::{merge_users, MergeLog};
use crate::users::User;
use actix_web::web;
use actix_web::{get, post, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct DirectMergeOptions {
    /// The primary user to merge into, whose data will be kept.
    pub primary_user: Uuid,
    /// The secondary user to merge, whose data will be merged into the other one.
    pub secondary_user: Uuid,
}

#[utoipa::path(
    post,
    summary = "[Staff]Direct merge",
    description = "Merges a user into another one. The primary user will keep their data, while the secondary user's data will be merged into the primary user. 
	This endpoint directly merges the users, without needing to go through a merge request.",
    tag = "Users - Merges",
    request_body = DirectMergeOptions,
    responses(
        (status = 200, body = User)
    ),
    security(
        ("access_token" = ["DirectMerge"]),
        ("api_key" = ["DirectMerge"]),
    )
)]
#[post("", wrap = "UserAuth::require(Permission::DirectMerge)")]
async fn direct_merge(
    db: web::Data<Arc<DbAppState>>,
    options: web::Json<DirectMergeOptions>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&options));
    let result = web::block(move || {
        merge_users(
            &mut db.connection()?,
            options.primary_user,
            options.secondary_user,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    get,
    summary = "[Staff]Get Merge logs",
    description = "Paginated logs of merged users",
    tag = "Users - Merges",
    params(
		("page" = Option<i64>, Query, description = "The page of the merge logs to fetch"),
		("per_page" = Option<i64>, Query, description = "The number of merge logs to fetch per page"),
	),
    responses(
        (status = 200, body = Paginated<MergeLog>)
    ),
    security(
        ("access_token" = ["MergeReview"]),
        ("api_key" = ["MergeReview"]),
    )
)]
#[get("/logs", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn list_logs(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<20>>,
) -> Result<HttpResponse, ApiError> {
    let result =
        web::block(move || MergeLogPage::find_all(&mut db.connection()?, page_query.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/requests", api = requests::ApiDoc)
    ),
    components(
        schemas(
            MergeLog,
			DirectMergeOptions
        )
    ),
    paths(
		direct_merge,
		list_logs
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/merge")
            .configure(requests::init_routes)
            .service(list_logs)
            .service(direct_merge),
    );
}
