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
use diesel::{pg::Pg, QueryDsl, RunQueryDsl, SelectableHelper, ExpressionMethods};


#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=aredl_submissions, check_for_backend(Pg))]
pub struct Submission {
    /// Internal UUID of the submission.
    pub id: Uuid,
    /// Level ID in the game. May not be unique for 2P levels.
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
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Whether the record was submitted as a priority record.
    pub priority: bool,
    
    pub is_update: bool,
    /// Whether this submission has been rejected.
    pub is_rejected: bool,
    /// The reason for rejecting this submission, if any.
    pub rejection_reason: Option<String>,
    /// Any additional notes left by the submitter.
    pub additional_notes: Option<String>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Insertable, Debug, ToSchema)]
#[diesel(table_name=aredl_submissions, check_for_backend(Pg))]
pub struct SubmissionInsert {
    /// UUID of the level this record is on.
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

impl Submission {
    pub fn find_all(db: web::Data<Arc<DbAppState>>) -> Result<Vec<Self>, ApiError> {
        let submissions = aredl_submissions::table
            .select(Submission::as_select())
            .load::<Self>(&mut db.connection()?)?;
        println!("{:?}", submissions);
        Ok(submissions)
    }
    
    pub fn create(db: web::Data<Arc<DbAppState>>, inserted_submission: SubmissionInsert) -> Result<Self, ApiError> {
        let submission = diesel::insert_into(aredl_submissions::table)
            .values(inserted_submission)
            .get_result::<Self>(&mut db.connection()?)?;

        Ok(submission)
    }

    pub fn delete(db: web::Data<Arc<DbAppState>>, submission_id: Uuid) -> Result<Submission, ApiError> {
        let deleted = diesel::delete(aredl_submissions::table)
            .filter(aredl_submissions::id.eq(submission_id))
            .get_result::<Self>(&mut db.connection()?)?;

        Ok(deleted)
    }
}
