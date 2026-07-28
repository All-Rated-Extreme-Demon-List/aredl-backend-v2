use crate::{
    app_data::db::DbAppState,
    auth::{Authenticated, Permission, UserAuth},
    error_handler::ApiError,
    shifts::{
        convert_start_hour_to_utc,
        recurring::{RecurringShift, RecurringShiftInsert, RecurringShiftPatch},
        ResolvedRecurringShift, SelfRecurringShiftInsert,
    },
};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "[Staff]List recurring shifts",
    description = "Get a possibly filtered list of the currently scheduled recurring shifts.",
    tag = "Shifts",
    responses(
        (status = 200, body = Vec<ResolvedRecurringShift>)
    ),
    security(
        ("access_token" = ["SubmissionReviewFull"]),
        ("api_key" = ["SubmissionReviewFull"]),
    ),
)]
#[get("", wrap = "UserAuth::require(Permission::SubmissionReviewFull)")]
async fn find_all_recurring_shifts(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let shifts = web::block(move || {
        ResolvedRecurringShift::find_all_for_user(&mut db.connection()?, &authenticated)
    })
    .await??;
    Ok(HttpResponse::Ok().json(shifts))
}

#[utoipa::path(
    post,
    summary = "[Staff]Create a recurring shift",
    description = "Schedules a new recurring shift for a user on a specific week day and time",
    tag = "Shifts",
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
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&body));
    let shift =
        web::block(move || RecurringShift::create(&mut db.connection()?, &body.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(shift))
}

#[utoipa::path(
    post,
    summary = "[Staff]Create own recurring shift",
    description = "Schedules a new recurring shift for the authenticated user.",
    tag = "Shifts",
    responses(
        (status = 200, body = RecurringShift)
    ),
    security(
        ("access_token" = ["ShiftCreateOwn"]),
        ("api_key" = ["ShiftCreateOwn"]),
    ),
)]
#[post("/@me", wrap = "UserAuth::require(Permission::ShiftCreateOwn)")]
async fn create_own_recurring_shift(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<SelfRecurringShiftInsert>,
    root_span: RootSpan,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&body));
    let shift = web::block(move || {
        let new_shift = body.into_inner();

        let start_hour_in_timezone: u32 = new_shift.start_hour.try_into().map_err(|_foo| {
            ApiError::BadRequest(
                "Invalid start hour provided. Please provide a valid hour between 0 and 23.",
            )
        })?;

        let timezone = new_shift
            .timezone
            .parse::<chrono_tz::Tz>()
            .map_err(|_foo| {
                ApiError::BadRequest(
                    "Invalid timezone provided. Please provide a valid IANA timezone string.",
                )
            })?;

        let (start_hour_utc, weekday) =
            convert_start_hour_to_utc(start_hour_in_timezone, &new_shift.weekday, timezone)?;

        RecurringShift::create(
            &mut db.connection()?,
            &RecurringShiftInsert {
                user_id: authenticated.user_id,
                start_hour: start_hour_utc.cast_signed(),
                weekday,
                duration: new_shift.duration,
                target_count: new_shift.target_count,
            },
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(shift))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit a recurring shift",
    description = "Edits a recurring shift data.",
    tag = "Shifts",
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
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&body));
    let updated = web::block(move || {
        RecurringShift::patch(&mut db.connection()?, id.into_inner(), &body.into_inner())
    })
    .await??;
    Ok(HttpResponse::Created().json(updated))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete a recurrent shift",
    description = "Deletes a recurrent shift.",
    tag = "Shifts",
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
    let deleted =
        web::block(move || RecurringShift::delete(&mut db.connection()?, id.into_inner()))
            .await??;
    Ok(HttpResponse::Created().json(deleted))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        ResolvedRecurringShift,
        RecurringShift,
        RecurringShiftPatch,
        SelfRecurringShiftInsert
    )),
    paths(
        find_all_recurring_shifts,
        patch_recurring_shift,
        delete_recurring_shift,
        create_new_recurring_shift,
        create_own_recurring_shift,
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/recurring")
            .service(find_all_recurring_shifts)
            .service(create_new_recurring_shift)
            .service(create_own_recurring_shift)
            .service(patch_recurring_shift)
            .service(delete_recurring_shift),
    );
}
