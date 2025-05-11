use crate::{
    arepl::shifts::{
        recurring::{RecurringShift, RecurringShiftInsert, RecurringShiftPatch},
        ResolvedRecurringShift,
    },
    auth::{Permission, UserAuth},
    db::DbAppState,
    error_handler::ApiError,
};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "[Staff]List recurring shifts",
    description = "Get a possibly filtered list of the currently scheduled recurring shifts.",
    tag = "AREDL (P) - Shifts",
    responses(
        (status = 200, body = Vec<ResolvedRecurringShift>)
    ),
    security(
        ("access_token" = ["ShiftManage"]),
        ("api_key" = ["ShiftManage"]),
    ),
)]
#[get("", wrap = "UserAuth::require(Permission::ShiftManage)")]
async fn find_all_recurring_shifts(
    db: web::Data<Arc<DbAppState>>,
) -> Result<HttpResponse, ApiError> {
    let shifts = web::block(move || ResolvedRecurringShift::find_all(&db)).await??;
    Ok(HttpResponse::Ok().json(shifts))
}

#[utoipa::path(
    post,
    summary = "[Staff]Create a recurring shift",
    description = "Schedules a new recurring shift for a user on a specific week day and time",
    tag = "AREDL (P) - Shifts",
    responses(
        (status = 200, body = RecurringShift)
    ),
    security(
        ("access_token" = ["ShiftManage"]),
        ("api_key" = ["ShiftManage"]),
    ),
)]
#[post("", wrap = "UserAuth::require(Permission::ShiftManage)")]
async fn create_new_recurring_shift(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<RecurringShiftInsert>,
) -> Result<HttpResponse, ApiError> {
    let shift = web::block(move || RecurringShift::create(&db, body.into_inner())).await??;
    Ok(HttpResponse::Ok().json(shift))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit a recurring shift",
    description = "Edits a recurring shift data.",
    tag = "AREDL (P) - Shifts",
    responses(
        (status = 200, body = RecurringShift)
    ),
	request_body = RecurringShiftPatch,
    security(
        ("access_token" = ["ShiftManage"]),
        ("api_key" = ["ShiftManage"]),
    )
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::ShiftManage)")]
async fn patch_recurring_shift(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<RecurringShiftPatch>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let updated =
        web::block(move || RecurringShift::patch(&db, id.into_inner(), body.into_inner()))
            .await??;
    Ok(HttpResponse::Created().json(updated))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete a recurrent shift",
    description = "Deletes a recurrent shift.",
    tag = "AREDL (P) - Shifts",
    responses(
        (status = 200, body = RecurringShift)
    ),
	params(
		("id" = Uuid, description = "The ID of the shift to delete"),
	),
    security(
        ("access_token" = ["ShiftManage"]),
        ("api_key" = ["ShiftManage"]),
    )
)]
#[delete("/{id}", wrap = "UserAuth::require(Permission::ShiftManage)")]
async fn delete_recurring_shift(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let deleted = web::block(move || RecurringShift::delete(&db, id.into_inner())).await??;
    Ok(HttpResponse::Created().json(deleted))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(ResolvedRecurringShift, RecurringShift, RecurringShiftPatch,)),
    paths(
        find_all_recurring_shifts,
        patch_recurring_shift,
        delete_recurring_shift,
        create_new_recurring_shift,
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/recurring")
            .service(find_all_recurring_shifts)
            .service(create_new_recurring_shift)
            .service(patch_recurring_shift)
            .service(delete_recurring_shift),
    );
}
