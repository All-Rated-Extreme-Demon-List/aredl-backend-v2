use uuid::Uuid;
use chrono::NaiveDateTime;
use actix_web::web;
use std::sync::Arc;
use crate::{
    schema::aredl_submissions, 
    db::DbAppState, 
    error_handler::ApiError
};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use diesel::{pg::Pg, QueryDsl, RunQueryDsl, SelectableHelper, Connection, ExpressionMethods};
use diesel_derive_enum::DbEnum;

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::SubmissionStatus"]
#[DbValueStyle = "PascalCase"]
pub enum SubmissionStatus {
    Pending,
    Claimed,
    UnderConsideration,
    Denied,
    Accepted
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Selectable, Debug, ToSchema)]
#[diesel(table_name = aredl_submissions, check_for_backend(Pg))]
pub struct Submission {
    /// Internal UUID of the submission.
    pub id: Uuid,
    /// UUID of the level this record is on.)
    pub level_id: Uuid,
    /// Internal UUID of the submitter.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The status of this submission
    pub status: SubmissionStatus,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Whether the record was submitted as a priority record.
    pub priority: bool,
    /// Whether this is a resubmission of an older record.
    pub is_update: bool,
    /// The reason for rejecting this submission, if any.
    pub rejection_reason: Option<String>,
    /// Any additional notes left by the submitter.
    pub additional_notes: Option<String>,
    /// Timestamp of when the submission was created.
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Insertable, ToSchema)]
#[diesel(table_name=aredl_submissions, check_for_backend(Pg))]
pub struct SubmissionInsert {
    /// UUID of the level this record is on.
    /// This will eventually resolve to a UUID.
    pub level_id: Uuid,
    /// Internal UUID of the submitter.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Any additional notes left by the submitter.
    pub additional_notes: Option<String>
}

#[derive(Serialize, Deserialize, Debug, AsChangeset)]
#[diesel(table_name=aredl_submissions, check_for_backend(Pg))]
pub struct SubmissionPatch {
    /// UUID of the level this record is on.)
    pub level_id: Option<Uuid>,
    /// Internal UUID of the submitter.
    pub submitted_by: Option<Uuid>,
    /// Whether the record was completed on mobile or not.
    pub mobile: Option<bool>,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: Option<String>,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The status of this submission
    pub status: Option<SubmissionStatus>,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// The reason for rejecting this submission, if any.
    pub rejection_reason: Option<String>,
    /// Any additional notes left by the submitter.
    pub additional_notes: Option<String>,
}

impl Submission {
    pub fn find_all(db: web::Data<Arc<DbAppState>>) -> Result<Vec<Self>, ApiError> {
        // TODO: paginate probably?
        let submissions = aredl_submissions::table
            .select(Self::as_select())
            .load::<Self>(&mut db.connection()?)?;
        Ok(submissions)
    }

    pub fn find_one(db: web::Data<Arc<DbAppState>>, id: Uuid) -> Result<Submission, ApiError> {
        let submissions = aredl_submissions::table
            .filter(aredl_submissions::id.eq(id))
            .select(Self::as_select())
            .first(&mut db.connection()?)?;
        Ok(submissions)
    }
    
    pub fn create(db: web::Data<Arc<DbAppState>>, inserted_submission: SubmissionInsert) -> Result<Self, ApiError> {
        let mut conn = db.connection()?;
        conn.transaction(|connection| -> Result<Self, ApiError> {
            diesel::insert_into(aredl_submissions::table)
                .values(&inserted_submission)
                .execute(connection)?;

            let submission = aredl_submissions::table
                .order(aredl_submissions::created_at.desc())
                .select(Self::as_select())
                .first::<Self>(connection)?;

            Ok(submission)
        })
    }

    pub fn delete(db: web::Data<Arc<DbAppState>>, submission_id: Uuid) -> Result<(), ApiError> {
        let mut conn = db.connection()?;
        conn.transaction(|connection| -> Result<(), ApiError> {
            diesel::delete(aredl_submissions::table)
				.filter(aredl_submissions::id.eq(submission_id))
				.execute(connection)?;
			Ok(())
		})?;
		Ok(())
    }
}

impl SubmissionPatch {
    pub fn patch(patch: Self, id: Uuid, db: web::Data<Arc<DbAppState>>) -> Result<Submission, ApiError> {
        let submission = diesel::update(aredl_submissions::table)
            .set(patch)
            .filter(aredl_submissions::id.eq(id))
            .returning(Submission::as_select())
            .get_result(&mut db.connection()?)?;
        Ok(submission)
    }
}
