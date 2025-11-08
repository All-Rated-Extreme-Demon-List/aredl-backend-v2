use crate::{
    aredl::submissions::{status::SubmissionsEnabled, *},
    auth::{Authenticated, Permission},
    db::DbConnection,
    error_handler::ApiError,
    schema::{
        aredl::{levels, submission_history, submissions},
        users,
    },
};
use diesel::expression_methods::BoolExpressionMethods;
use diesel::{
    dsl::exists, select, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use is_url::is_url;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::history::SubmissionHistory;

#[derive(Serialize, Deserialize, Debug, AsChangeset, Default, ToSchema, Clone, PartialEq)]
#[diesel(table_name=submissions, check_for_backend(Pg))]
pub struct SubmissionPatchUser {
    /// UUID of the level this record is on.)
    pub level_id: Option<Uuid>,
    /// Whether the record was completed on mobile or not.
    pub mobile: Option<bool>,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: Option<String>,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, AsChangeset, Default, ToSchema, Clone, PartialEq)]
#[diesel(table_name=submissions, check_for_backend(Pg))]
pub struct SubmissionPatchMod {
    /// UUID of the level this record is on.)
    pub level_id: Option<Uuid>,
    /// [Mod only] Internal UUID of the submitter.
    pub submitted_by: Option<Uuid>,
    /// Whether the record was completed on mobile or not.
    pub mobile: Option<bool>,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: Option<String>,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// [Mod only] The status of the submission
    pub status: Option<SubmissionStatus>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// Whether the record was submitted as a priority record.
    pub priority: Option<bool>,
    /// [Mod only] Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// [Mod only] Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
}

impl SubmissionPatchUser {
    pub fn patch(
        patch: Self,
        id: Uuid,
        conn: &mut DbConnection,
        authenticated: Authenticated,
    ) -> Result<Submission, ApiError> {
        let user = authenticated.user_id;
        if patch == Self::default() {
            return Err(ApiError::new(400, "No changes were provided in the patch!"));
        }

        if let Some(video_url) = patch.video_url.as_ref() {
            if !is_url(video_url) {
                return Err(ApiError::new(400, "Your video is not a URL!"));
            }
        }

        if let Some(raw_url) = patch.raw_url.as_ref() {
            if !is_url(raw_url) {
                return Err(ApiError::new(400, "Your raw footage is not a URL!"));
            }
        }

        let old_submission = submissions::table
            .filter(submissions::id.eq(id))
            .select(Submission::as_select())
            .first::<Submission>(conn)?;

        let resub = old_submission.status == SubmissionStatus::Denied;

        if resub {
            if !SubmissionsEnabled::is_enabled(conn)? {
                return Err(ApiError::new(
                    400,
                    "Submissions are closed, please wait to resubmit this record!",
                ));
            }
            let submitter_ban = users::table
                .filter(users::id.eq(user))
                .select(users::ban_level)
                .first::<i32>(conn)?;

            if submitter_ban >= 2 {
                return Err(ApiError::new(
                    403,
                    "You are banned from resubmitting records.",
                ));
            }
        }

        let level_id = match patch.level_id {
            Some(new_level_id) => new_level_id,
            None => old_submission.level_id,
        };

        let raw_footage = match patch.raw_url.clone() {
            Some(raw) => Some(raw),
            None => old_submission.raw_url,
        };

        // if either of these fields change, we need to revalidate the raw
        if patch.level_id.is_some() || patch.raw_url.is_some() {
            let level_exists = levels::table
                .filter(levels::id.eq(level_id))
                .select((levels::legacy, levels::position))
                .first::<(bool, i32)>(conn)
                .optional()?;

            match level_exists {
                None => return Err(ApiError::new(404, "Could not find the new level!")),
                Some((is_legacy, pos)) => {
                    if is_legacy == true {
                        return Err(ApiError::new(
                            400,
                            "This level is on the legacy list, and is not accepting records!",
                        ));
                    }
                    if pos < 400 && raw_footage.is_none() {
                        return Err(ApiError::new(
                            400,
                            "This level is top 400 and requires raw footage!",
                        ));
                    }
                }
            }
        }

        let existing_submission = submissions::table
            .filter(submissions::level_id.eq(level_id))
            .filter(submissions::submitted_by.eq(old_submission.submitted_by))
            .filter(submissions::id.ne(id))
            .select(submissions::id)
            .first::<Uuid>(conn)
            .optional()?;

        if existing_submission.is_some() {
            return Err(ApiError::new(
                409,
                "This user already has a submission for this level!",
            ));
        }

        let mut result = diesel::update(submissions::table)
            .filter(submissions::id.eq(id))
            .filter(submissions::submitted_by.eq(user))
            .filter(
                submissions::status
                    .eq(SubmissionStatus::Pending)
                    .or(submissions::status.eq(SubmissionStatus::Denied)),
            )
            .set((
                patch.clone(),
                // FIXME: this is very silly
                if resub {
                    (
                        submissions::status.eq(SubmissionStatus::Pending),
                        submissions::reviewer_id.eq::<Option<Uuid>>(None),
                        submissions::reviewer_notes.eq::<Option<String>>(None),
                    )
                } else {
                    (
                        submissions::status.eq(old_submission.status),
                        submissions::reviewer_id.eq(old_submission.reviewer_id),
                        submissions::reviewer_notes.eq(old_submission.reviewer_notes),
                    )
                },
            ))
            .returning(Submission::as_select())
            .get_result::<Submission>(conn)?;

        let history = SubmissionHistory {
            id: Uuid::new_v4(),
            submission_id: result.id,
            record_id: None,
            status: SubmissionStatus::Pending,
            user_notes: result.user_notes.clone(),
            reviewer_id: None,
            reviewer_notes: None,
            timestamp: chrono::Utc::now(),
        };

        diesel::insert_into(submission_history::table)
            .values(&history)
            .execute(conn)?;

        if !authenticated.has_permission(conn, Permission::SubmissionReview)? {
            result.reviewer_id = None;
        }

        Ok(result)
    }
}

impl SubmissionPatchMod {
    pub fn patch_mod(
        patch: Self,
        id: Uuid,
        conn: &mut DbConnection,
        authenticated: Authenticated,
    ) -> Result<Submission, ApiError> {
        if patch == Self::default() {
            return Err(ApiError::new(400, "No changes were provided in the patch!"));
        }

        if let Some(video_url) = patch.video_url.as_ref() {
            if !is_url(video_url) {
                return Err(ApiError::new(400, "Your video is not a URL!"));
            }
        }

        if let Some(raw_url) = patch.raw_url.as_ref() {
            if !is_url(raw_url) {
                return Err(ApiError::new(400, "Your raw footage is not a URL!"));
            }
        }

        let old_submission = submissions::table
            .filter(submissions::id.eq(id))
            .select(Submission::as_select())
            .first::<Submission>(conn)?;

        if old_submission.submitted_by == authenticated.user_id {
            return SubmissionPatchUser::patch(
                SubmissionPatchMod::downgrade(patch),
                id,
                conn,
                authenticated,
            );
        }

        let level_id = match patch.level_id {
            Some(new_level_id) => new_level_id,
            None => old_submission.level_id,
        };

        let submitted_by = match patch.submitted_by {
            Some(new_submitter_id) => new_submitter_id,
            None => old_submission.submitted_by,
        };

        if let Some(new_submitter) = patch.submitted_by {
            let submitter_ban = users::table
                .filter(users::id.eq(new_submitter))
                .select(users::ban_level)
                .first::<i32>(conn)
                .optional()?;

            match submitter_ban {
                None => return Err(ApiError::new(404, "Could not find the new user!")),
                Some(ban) => {
                    if ban >= 2 {
                        return Err(ApiError::new(403, "This user is submission banned!"));
                    }
                }
            }
        }

        if let Some(new_level) = patch.level_id {
            let level_exists = select(exists(levels::table.filter(levels::id.eq(new_level))))
                .get_result::<bool>(conn)?;

            if level_exists == false {
                return Err(ApiError::new(404, "Could not find the new level!"));
            }
        }

        let existing_submission = submissions::table
            .filter(submissions::level_id.eq(level_id))
            .filter(submissions::submitted_by.eq(submitted_by))
            .filter(submissions::id.ne(id))
            .select(submissions::id)
            .first::<Uuid>(conn)
            .optional()?;

        if existing_submission.is_some() {
            return Err(ApiError::new(
                409,
                "This user already has a submission for this level!",
            ));
        }

        let result = diesel::update(submissions::table)
            .filter(submissions::id.eq(id))
            .set(patch.clone())
            .returning(Submission::as_select())
            .get_result::<Submission>(conn)?;

        Ok(result)
    }

    pub fn downgrade(s: Self) -> SubmissionPatchUser {
        SubmissionPatchUser {
            level_id: s.level_id,
            mobile: s.mobile,
            ldm_id: s.ldm_id,
            video_url: s.video_url,
            raw_url: s.raw_url,
            mod_menu: s.mod_menu,
            user_notes: s.user_notes,
        }
    }
}
