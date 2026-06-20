use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    app_data::db::DbAppState,
    auth::{connected_accounts::model::OAuthConnectedAccount, Authenticated, UserAuth},
    error_handler::ApiError,
};

#[utoipa::path(
    get,
    summary = "[Auth]Get all connected accounts for a user",
    description = "Returns all connected accounts for a specific user.",
    tag = "Authentication",
    responses(
        (status = 200, description = "Connected accounts", body = Vec<OAuthConnectedAccount>),
        (status = 404, description = "User not found"),

    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("user_id" = Uuid, description = "The ID of the user for whom to retrieve connected accounts")
    )
)]
#[get("/user/{user_id}", wrap = "UserAuth::load()")]
async fn get_connected_accounts(
    db: web::Data<Arc<DbAppState>>,
    user_id: web::Path<Uuid>,
    auth: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let connected_accounts = web::block(move || {
        OAuthConnectedAccount::find_all_by_user_id(
            &mut db.connection()?,
            user_id.into_inner(),
            &auth,
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(connected_accounts))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(OAuthConnectedAccount)),
    paths(get_connected_accounts,)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/connected-accounts").service(get_connected_accounts));
}
