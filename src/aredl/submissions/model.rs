use uuid::Uuid;
use chrono::NaiveDateTime;
use actix_web::web;
use std::sync::Arc;
use crate::{
    db::DbAppState, 
    error_handler::ApiError, 
    schema::{
        aredl_submissions, aredl_submissions_with_priority
    }
};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use diesel::{
    pg::Pg, 
    QueryDsl, 
    RunQueryDsl, 
    SelectableHelper,
    Connection, 
    ExpressionMethods, 
    JoinOnDsl
};
use diesel_derive_enum::DbEnum;
use crate::aredl::levels::records::{RecordInsert, Record};

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
    
    // this is an Option so it's possible to exclude it from
    // the request body without throwing an error
    pub mobile: Option<bool>,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Any additional notes left by the submitter.
    pub additional_notes: Option<String>,
    // not documented, this will be resolved
    // automatically in the future
    pub priority: Option<bool>
}

#[derive(Serialize, Deserialize, Debug, AsChangeset, Default)]
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

#[derive(Serialize, Deserialize)]
pub struct RejectionData {
    /// The reason for rejecting this record
    pub reason: Option<String>
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
        let submission = aredl_submissions::table
            .filter(aredl_submissions::id.eq(id))
            .select(Self::as_select())
            .first(&mut db.connection()?)?;
        Ok(submission)
    }

    pub fn find_highest_priority(db: web::Data<Arc<DbAppState>>, user: Uuid) -> Result<Submission, ApiError> {
        let _new_data = SubmissionPatch {
            reviewer_id: Some(user),
            status: Some(SubmissionStatus::Claimed),
            ..Default::default()
        };
        // TODO: edit data
        let submission = aredl_submissions::table
            .inner_join(aredl_submissions_with_priority::table.on(aredl_submissions::id.eq(aredl_submissions_with_priority::id)))
            .order(aredl_submissions_with_priority::priority_value.desc())
            .limit(1)
            .select(Self::as_select())
            .first(&mut db.connection()?)?;
        Ok(submission)
    }
    
    pub fn create(db: web::Data<Arc<DbAppState>>, inserted_submission: SubmissionInsert) -> Result<Self, ApiError> {
        let mut conn = db.connection()?;
        conn.transaction(|connection| -> Result<Self, ApiError> {
            let submission = diesel::insert_into(aredl_submissions::table)
                .values(&inserted_submission)
                .returning(Self::as_select())
                .get_result(connection)?;

            Ok(submission)
        })
    }

    pub fn delete(db: web::Data<Arc<DbAppState>>, submission_id: Uuid) -> Result<(), ApiError> {
        let mut conn = db.connection()?;
        conn.transaction(|connection| -> Result<Submission, ApiError> {
            let deleted = diesel::delete(aredl_submissions::table)
				.filter(aredl_submissions::id.eq(submission_id))
                .returning(Self::as_select())
				.get_result(connection)?;
			Ok(deleted)
		})?;
		Ok(())
    }

    pub fn accept(db: web::Data<Arc<DbAppState>>, id: Uuid, reviewer_id: Uuid) -> Result<Record, ApiError> {
        let conn = &mut db.connection()?;
        let new_data = SubmissionPatch {
            status: Some(SubmissionStatus::Accepted),
            reviewer_id: Some(reviewer_id),
            ..Default::default()
        };

        let updated: Submission = diesel::update(aredl_submissions::table)
            .filter(aredl_submissions::id.eq(id))
            .set(new_data)
            .returning(Self::as_select())
            .get_result(conn)?;

        let record = RecordInsert {
            submitted_by: updated.submitted_by,
            mobile: updated.mobile,
            ldm_id: updated.ldm_id,
            video_url: updated.video_url,
            raw_url: updated.raw_url,
            reviewer_id: Some(reviewer_id),
            created_at: Some(updated.created_at),
            updated_at: None
        };

        let inserted = Record::create(db, updated.level_id, record)?;

        Ok(inserted)
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
