use crate::{
    aredl::shifts::{recurring, Shift, ShiftFilterQuery, ShiftPage, ShiftPatch, ShiftStatus},
    auth::{Authenticated, Permission, UserAuth},
    db::DbAppState,
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
};
use actix_web::{delete, get, patch, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "[Staff]List shifts",
    description = "Get a possibly filtered list of all current and past shifts.",
    tag = "AREDL - Shifts",
    responses(
        (status = 200, body = Paginated<ShiftPage>)
    ),
	params(
		("page" = i64, description = "The page number to fetch"),
		("per_page" = i64, description = "The number of items per page"),
		("status" = ShiftStatus, description = "The status of the shifts to fetch"),
		("user_id" = Uuid, description = "The ID of the user to filter by"),
	),
    security(
        ("access_token" = ["ShiftManage"]),
        ("api_key" = ["ShiftManage"]),
    ),
)]
#[get("", wrap = "UserAuth::require(Permission::ShiftManage)")]
async fn find_all_shifts(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<50>>,
    options: web::Query<ShiftFilterQuery>,
) -> Result<HttpResponse, ApiError> {
    let shifts =
        web::block(move || ShiftPage::find_all(&db, page_query.into_inner(), options.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(shifts))
}

#[utoipa::path(
    get,
    summary = "[Staff]List my shifts",
    description = "Get a list of all current and past shifts for the authenticated user.",
    tag = "AREDL - Shifts",
    responses(
        (status = 200, body = Paginated<ShiftPage>)
    ),
	params(
		("page" = i64, description = "The page number to fetch"),
		("per_page" = i64, description = "The number of items per page"),
	),
    security(
        ("access_token" = ["ShiftManage"]),
        ("api_key" = ["ShiftManage"]),
    ),
)]
#[get("@/me", wrap = "UserAuth::require(Permission::SubmissionReview)")]
async fn find_all_shifts_me(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<50>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let shifts =
        web::block(move || ShiftPage::find_me(&db, page_query.into_inner(), authenticated.user_id))
            .await??;
    Ok(HttpResponse::Ok().json(shifts))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit a shift",
    description = "Edits a current or past shift.",
    tag = "AREDL - Shifts",
    responses(
        (status = 200, body = Shift)
    ),
	request_body = ShiftPatch,
    security(
        ("access_token" = ["ShiftManage"]),
        ("api_key" = ["ShiftManage"]),
    )
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::ShiftManage)")]
async fn patch_shift(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<ShiftPatch>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let updated =
        web::block(move || Shift::patch(&db, id.into_inner(), body.into_inner())).await??;
    Ok(HttpResponse::Created().json(updated))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete a shift",
    description = "Deletes a current or past shift.",
    tag = "AREDL - Shifts",
    responses(
        (status = 200, body = Shift)
    ),
    security(
        ("access_token" = ["ShiftManage"]),
        ("api_key" = ["ShiftManage"]),
    )
)]
#[delete("/{id}", wrap = "UserAuth::require(Permission::ShiftManage)")]
async fn delete_shift(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let deleted = web::block(move || Shift::delete(&db, id.into_inner())).await??;
    Ok(HttpResponse::Created().json(deleted))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Shifts", description = "Endpoints for fetching and managing shifts and recurring shifts."),
    ),
    nest(
        (path = "/recurring", api=recurring::ApiDoc),
    ),
    components(
        schemas(
            Shift,
			ShiftPatch,
			ShiftPage,
			ShiftFilterQuery,
        )
    ),
    paths(
        find_all_shifts,
        find_all_shifts_me,
		patch_shift,
		delete_shift,
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/shifts")
            .configure(recurring::init_routes)
            .service(find_all_shifts)
            .service(find_all_shifts_me)
            .service(patch_shift)
            .service(delete_shift),
    );
}
