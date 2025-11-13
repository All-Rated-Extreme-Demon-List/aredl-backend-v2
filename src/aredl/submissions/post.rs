use crate::{
    aredl::submissions::{history::SubmissionHistory, status::SubmissionsEnabled, *},
    auth::Authenticated,
    app_data::db::DbConnection,
    error_handler::ApiError,
    roles::Role,
    schema::{
        aredl::{levels, submission_history, submissions},
        roles, user_roles,
    },
};
use diesel::{
    Connection, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use is_url::is_url;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Insertable, ToSchema)]
#[diesel(table_name=submissions, check_for_backend(Pg))]
// this struct does not contain the player's ID, which is computed to
// be the logged in user. thus, this struct cannot be and is not inserted directly
// into the query. if a new property is added here, remember to update Submission::create()
// to insert that property into the database!

// TODO: rework this probably
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
        conn: &mut DbConnection,
        inserted_submission: SubmissionInsert,
        authenticated: Authenticated,
    ) -> Result<Self, ApiError> {
        if !is_url(&inserted_submission.video_url) {
            return Err(ApiError::new(400, "Your completion link is not a URL"));
        }

        if let Some(raw_url) = inserted_submission.raw_url.as_ref() {
            if !is_url(raw_url) {
                return Err(ApiError::new(400, "Your raw footage is not a URL"));
            }
        }

        conn.transaction(|connection| -> Result<Self, ApiError> {
            // a bunch of validation yay

            // check if submissions are disabled
            if !(SubmissionsEnabled::is_enabled(connection)?) {
                return Err(ApiError::new(400, "Submissions are currently disabled"));
            }

            // check if any submissions exist already
            let exists_submission = submissions::table
                .filter(submissions::submitted_by.eq(authenticated.user_id))
                .filter(submissions::level_id.eq(inserted_submission.level_id))
                .select(submissions::id)
                .first::<Uuid>(connection)
                .optional()?;

            if exists_submission.is_some() {
                return Err(ApiError::new(
                    409,
                    "You already have a submission for this level",
                ));
            }

            // check that this level exists, is not legacy, and
            // raw footage is provided for ranks 400+
            let level_info = levels::table
                .filter(levels::id.eq(inserted_submission.level_id))
                .select((levels::legacy, levels::position))
                .first::<(bool, i32)>(connection)
                .optional()?;

            match level_info {
                None => return Err(ApiError::new(404, "Could not find this level")),
                Some((legacy, pos)) => {
                    if legacy == true {
                        return Err(ApiError::new(
                            400,
                            "This level is on the legacy list and is not accepting records.",
                        ));
                    }
                    if pos <= 400 && inserted_submission.raw_url.is_none() {
                        return Err(ApiError::new(
                            400,
                            "This level is top 400 and requires raw footage",
                        ));
                    }
                }
            }

            let roles = user_roles::table
                .inner_join(roles::table.on(user_roles::role_id.eq(roles::id)))
                .filter(user_roles::user_id.eq(authenticated.user_id))
                .select(Role::as_select())
                .load::<Role>(connection)?;

            let has_role = roles.iter().any(|role| role.privilege_level == 5);

            let submission = diesel::insert_into(submissions::table)
                .values((
                    submissions::submitted_by.eq(authenticated.user_id),
                    submissions::level_id.eq(inserted_submission.level_id),
                    inserted_submission.mobile.map_or_else(
                        || submissions::mobile.eq(false),
                        |mobile| submissions::mobile.eq(mobile),
                    ),
                    submissions::ldm_id.eq(inserted_submission.ldm_id),
                    submissions::video_url.eq(inserted_submission.video_url),
                    submissions::raw_url.eq(inserted_submission.raw_url),
                    submissions::mod_menu.eq(inserted_submission.mod_menu),
                    submissions::user_notes.eq(inserted_submission.user_notes),
                    submissions::priority.eq(has_role),
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
