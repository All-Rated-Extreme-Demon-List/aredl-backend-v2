use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, UserAuth};
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::records::Record;
use crate::list::submissions::sync::{RecordsSync, RecordsSyncProvider};
use crate::list::submissions::Submission;
use crate::list::List;
use crate::providers::ProvidersAppState;
use crate::users::User;
use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

#[derive(serde::Deserialize, ToSchema)]
struct RecordSyncPath {
    list: List,
    provider: RecordsSyncProvider,
}

#[utoipa::path(
    post,
    summary = "[Auth]Sync records",
    description = "Import and/or update submissions from another list.",
    tag = "List - Submissions",
    params(
        ("list" = List, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("provider" = RecordsSyncProvider, description = "The external list from which to import records from")
    ),
    responses(
        (status = 200, body = Vec<Submission>),
        (status = 404, description = "User or Pemonlist player not found", body = ErrorResponse),
        (status = 422, description = "Your account is not linked to a Discord account", body = ErrorResponse),
        (status = 502, description = "Failed to request Pemonlist data, parse its response or parse a completion time", body = ErrorResponse, examples(
            ("request_failed" = (value = json!({"message": "error sending request for url (https://pemonlist.com/api/player/123456789)"}))),
            ("invalid_response" = (value = json!({"message": "Failed to parse data received from pemonlist: error decoding response body"}))),
            ("invalid_completion_time" = (value = json!({"message": "Malformed formatted_time (out of range)"})))
        ))
    ),
    security(("bearer_token" = [])),
)]
#[post("/{provider}", wrap = "UserAuth::load()")]
async fn sync(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
    path: web::Path<RecordSyncPath>,
    providers: web::Data<Arc<ProvidersAppState>>,
) -> Result<HttpResponse, ApiError> {
    let path = path.into_inner();

    let result = web::block(move || {
        let user = User::from_uuid(&mut db.connection()?, authenticated.user_id)?;

        let records_to_import = RecordsSync::sync(path.provider, path.list, &user)?;

        let result = RecordsSync::import_records(
            &mut db.connection()?,
            path.provider,
            path.list,
            &user,
            records_to_import,
        )?;

        Record::post_accept_actions(db, path.list, &result.changed_submissions, providers);

        Ok::<_, ApiError>(result)
    })
    .await??;

    Ok(HttpResponse::Ok().json(result.submissions))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "List - Submissions", description = "Endpoints for fetching and managing platformer submissions")
    ),
    paths(
		sync
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/sync").service(sync));
}
