use crate::{
    aredl::submissions::*,
    auth::Authenticated,
    db::DbAppState,
    error_handler::ApiError,
    roles::Role,
    schema::{aredl_levels, aredl_submissions, roles, submission_history, user_roles, users},
};
use actix_web::web;
use diesel::{
    Connection, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use is_url::is_url;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use super::history::SubmissionHistory;

#[derive(Serialize, Deserialize, Debug, Insertable, ToSchema)]
#[diesel(table_name=aredl_submissions, check_for_backend(Pg))]
// this struct does not contain the player's ID, which is computed to
// be the logged in user. thus, this struct cannot be and is not inserted directly
// to insert that property into the database!
// into the query. if a new property is added here, remember to update Submission::create()
pub struct SubmissionInsert {
    /// UUID of the level this record is on.
    pub level_id: Uuid,
    /// Set to `true` if this completion is on a mobile device.
    pub mobile: Option<bool>,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
}

impl Submission {
    pub fn create(
        db: web::Data<Arc<DbAppState>>,
        inserted_submission: SubmissionInsert,
        authenticated: Authenticated,
    ) -> Result<Self, ApiError> {
        let mut conn = db.connection()?;

        if !is_url(&inserted_submission.video_url) {
            return Err(ApiError::new(400, "Your completion link is not a URL!"));
        }

        if let Some(raw_url) = inserted_submission.raw_url.as_ref() {
            if !is_url(raw_url) {
                return Err(ApiError::new(400, "Your raw footage is not a URL!"));
            }
        }

        conn.transaction(|connection| -> Result<Self, ApiError> {
            // a bunch of validation yay

            // check if any submissions exist already
            let exists_submission = aredl_submissions::table
                .filter(aredl_submissions::submitted_by.eq(authenticated.user_id))
                .filter(aredl_submissions::level_id.eq(inserted_submission.level_id))
                .select(aredl_submissions::id)
                .first::<Uuid>(connection)
                .optional()?;

            if exists_submission.is_some() {
                return Err(ApiError::new(
                    409,
                    "You already have a submission for this level!",
                ));
            }

            // check that this level exists, is not legacy, and
            // raw footage is provided for ranks 400+
            let level_info = aredl_levels::table
                .filter(aredl_levels::id.eq(inserted_submission.level_id))
                .select((aredl_levels::legacy, aredl_levels::position))
                .first::<(bool, i32)>(connection)
                .optional()?;

            match level_info {
                None => return Err(ApiError::new(404, "Could not find this level!")),
                Some((legacy, pos)) => {
                    if legacy == true {
                        return Err(ApiError::new(
                            400,
                            "This level is on the legacy list and is not accepting records!",
                        ));
                    }
                    if pos <= 400 && inserted_submission.raw_url.is_none() {
                        return Err(ApiError::new(
                            400,
                            "This level is top 400 and requires raw footage!",
                        ));
                    }
                }
            }

            // check that this user ID is not banned
            // we know this user exists because it's based on the
            // authenticated user
            let submitter_ban = users::table
                .filter(users::id.eq(&authenticated.user_id))
                .select(users::ban_level)
                .first::<i32>(connection)?;

            if submitter_ban >= 2 {
                return Err(ApiError::new(
                    403,
                    "You are banned from submitting records.",
                ));
            }

            let roles = user_roles::table
                .inner_join(roles::table.on(user_roles::role_id.eq(roles::id)))
                .filter(user_roles::user_id.eq(authenticated.user_id))
                .select(Role::as_select())
                .load::<Role>(connection)?;

            let has_role = roles.iter().any(|role| role.privilege_level == 5);

            let submission = diesel::insert_into(aredl_submissions::table)
                .values((
                    aredl_submissions::submitted_by.eq(authenticated.user_id),
                    aredl_submissions::level_id.eq(inserted_submission.level_id),
                    inserted_submission.mobile.map_or_else(
                        || aredl_submissions::mobile.eq(false),
                        |mobile| aredl_submissions::mobile.eq(mobile),
                    ),
                    aredl_submissions::ldm_id.eq(inserted_submission.ldm_id),
                    aredl_submissions::video_url.eq(inserted_submission.video_url),
                    aredl_submissions::raw_url.eq(inserted_submission.raw_url),
                    aredl_submissions::mod_menu.eq(inserted_submission.mod_menu),
                    aredl_submissions::user_notes.eq(inserted_submission.user_notes),
                    aredl_submissions::priority.eq(has_role),
                ))
                .returning(Self::as_select())
                .get_result(connection)?;

            // Log submission creation history
            let history = SubmissionHistory {
                id: Uuid::new_v4(),
                submission_id: submission.id,
                record_id: None,
                status: SubmissionStatus::Pending,
                user_notes: submission.user_notes.clone(),
                reviewer_id: None,
                reviewer_notes: None,
                timestamp: chrono::Utc::now(),
            };

            diesel::insert_into(submission_history::table)
                .values(&history)
                .execute(connection)?;

            Ok(submission)
        })
    }
}
