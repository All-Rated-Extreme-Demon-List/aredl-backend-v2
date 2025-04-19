use uuid::Uuid;
use chrono::NaiveDateTime;
use actix_web::web;
use std::sync::Arc;
use crate::{
    db::DbAppState, error_handler::ApiError, page_helper::Paginated, schema::{
        aredl_levels, aredl_records, aredl_submissions, aredl_submissions_with_priority, users
    }
};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use diesel::{
    pg::Pg, r2d2::{
        ConnectionManager, PooledConnection
    }, 
    sql_types::Bool,
    Connection, 
    ExpressionMethods, 
    PgConnection, 
    QueryDsl, 
    RunQueryDsl, 
    SelectableHelper, 
    OptionalExtension,
    BoxableExpression,
    IntoSql
};
use diesel_derive_enum::DbEnum;
use crate::{
    aredl::levels::{
        BaseLevel,
        ResolvedLevel,
        records::{RecordInsert, Record}
    },
    users::{
        BaseUser,
        me::notifications::{Notification, NotificationType}
    },
    auth::{Authenticated, Permission},
    page_helper::PageQuery
};

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

#[derive(Serialize, Deserialize)]
pub struct BaseSubmission {
    /// Internal UUID of the submission.
    pub id: Uuid,
    /// Name of the level this submission is for.
    pub level: String,
    /// The submitter's name
    pub submitter: String,
    /// The status of this submission
    pub status: SubmissionStatus,
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

#[derive(Serialize, Deserialize, Queryable, Insertable, Selectable, Debug, ToSchema)]
#[diesel(table_name = aredl_submissions_with_priority, check_for_backend(Pg))]
pub struct SubmissionWithPriority {
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
    /// The priority value of this submission
    pub priority_value: i64,
}

#[derive(Serialize, Deserialize)]
pub struct SubmissionResolved {
    /// Internal UUID of the submission.
    pub id: Uuid,
    /// UUID of the level this record is on.)
    pub level: BaseLevel,
    /// Internal UUID of the submitter.
    pub submitted_by: BaseUser,
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
    pub reviewer: Option<BaseUser>,
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
    /// 
    pub priority_value: i64
}

#[derive(Serialize, Deserialize, Debug, Insertable, ToSchema)]
#[diesel(table_name=aredl_submissions, check_for_backend(Pg))]
// this struct does not contain the player's ID, which is computed to
// be the logged in user. thus, this struct cannot be and is not inserted directly
// into the query. if a new property is added here, remember to update Submission::create()
// to insert that property into the database!
pub struct SubmissionInsert {
    /// UUID of the level this record is on.
    /// This will eventually resolve to a UUID.
    pub level_id: Uuid,
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
pub struct SubmissionQueue {
    /// The amount of pending submissions in the database.
    pub levels_in_queue: i32
}

#[derive(Serialize, Deserialize)]
pub struct RejectionData {
    /// The reason for rejecting this record
    pub reason: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct SubmissionPage {
    data: Vec<Submission>
}

#[derive(Serialize, Deserialize)]
pub struct SubmissionQueryOptions {
    pub status_filter: Option<SubmissionStatus>,
    pub mobile_fiter: Option<bool>,
    pub level_filter: Option<Uuid>,
    pub submitter_filter: Option<Uuid>,
    pub priority_filter: Option<bool>,
}

impl Submission {
    pub fn create(db: web::Data<Arc<DbAppState>>, inserted_submission: SubmissionInsert, authenticated: Authenticated) -> Result<Self, ApiError> {
        let mut conn = db.connection()?;
        conn.transaction(|connection| -> Result<Self, ApiError> {
            let exists = aredl_records::table
                .filter(aredl_records::submitted_by.eq(authenticated.user_id))
                .filter(aredl_records::level_id.eq(inserted_submission.level_id))
                .select(aredl_records::id)
                .first::<Uuid>(connection)
                .optional()?;

            if exists.is_some() {
                return Err(ApiError::new(409, "This user already has a record on this level."));
            }
            let submission = diesel::insert_into(aredl_submissions::table)
                .values((
                    // this is fun
                    aredl_submissions::submitted_by.eq(authenticated.user_id),
                    aredl_submissions::level_id.eq(inserted_submission.level_id),
                    // what the fuck
                    inserted_submission.mobile.map_or_else(
                        || aredl_submissions::mobile.eq(false),
                        |mobile| aredl_submissions::mobile.eq(mobile)
                    ),
                    aredl_submissions::ldm_id.eq(inserted_submission.ldm_id),
                    aredl_submissions::video_url.eq(inserted_submission.video_url),
                    aredl_submissions::raw_url.eq(inserted_submission.raw_url),
                    aredl_submissions::additional_notes.eq(inserted_submission.additional_notes),
                    // what the fuck part 2
                    inserted_submission.priority.map_or_else(
                        || aredl_submissions::priority.eq(false),
                        |priority| aredl_submissions::priority.eq(priority)
                    )
                ))
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
        conn.transaction(|connection| -> Result<Record, ApiError> {
            let new_data = SubmissionPatch {
                status: Some(SubmissionStatus::Accepted),
                reviewer_id: Some(reviewer_id),
                ..Default::default()
            };

            let updated: Submission = diesel::update(aredl_submissions::table)
                .filter(aredl_submissions::id.eq(id))
                .set(new_data)
                .returning(Self::as_select())
                .get_result(connection)?;

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

            let level_name = aredl_levels::table
                .filter(aredl_levels::id.eq(updated.level_id))
                .select(aredl_levels::name)
                .first::<String>(connection)?;

            let content = format!("Your record on {:?} has been accepted!", level_name);
            Notification::create(
                connection, 
                inserted.submitted_by, 
                content, 
                NotificationType::Success
            )?;
            Ok(inserted)
        })
    }
    pub fn reject(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid, 
        authenticated: Authenticated,
        reason: Option<String>
    ) -> Result<SubmissionResolved, ApiError> { 
        let connection = &mut db.connection()?;
        let new_data = SubmissionPatch {
            status: Some(SubmissionStatus::Denied),
            reviewer_id: Some(authenticated.user_id),
            rejection_reason: reason,
            ..Default::default()
        };

        let new_record = SubmissionPatch::patch(new_data, id, &mut db.connection()?, true, authenticated.user_id)?;

        let upgraded = SubmissionResolved::from(new_record, db, None)?;
        
        let content = format!("Your record on {:?} has been denied...", upgraded.level.name);
        Notification::create(
            connection, 
            upgraded.submitted_by.id, 
            content, 
            NotificationType::Failure
        )?;
        Ok(upgraded)
    }
    pub fn under_consideration(db: web::Data<Arc<DbAppState>>, id: Uuid, authenticated: Authenticated) -> Result<SubmissionResolved, ApiError> {
        let connection = &mut db.connection()?;
        let new_data = SubmissionPatch {
            status: Some(SubmissionStatus::UnderConsideration),
            reviewer_id: Some(authenticated.user_id),
            ..Default::default()
        };

        let new_record = SubmissionPatch::patch(new_data, id, connection, true, authenticated.user_id)?;

        let upgraded = SubmissionResolved::from(new_record, db, None)?;
        
        let content = format!("Your record on {:?} has been placed under consideration.", upgraded.level.name);
        Notification::create(
            connection,
            upgraded.submitted_by.id,
            content, 
            NotificationType::Info
        )?;
        Ok(upgraded)

    }
}

impl SubmissionPatch {
    pub fn patch(patch: Self, id: Uuid, conn: &mut PooledConnection<ConnectionManager<PgConnection>>, has_auth: bool, user: Uuid) -> Result<Submission, ApiError> {
        let mut query = diesel::update(aredl_submissions::table)
            .filter(aredl_submissions::id.eq(id))
            .set(patch)
            .returning(Submission::as_select())
            .into_boxed();

        if !has_auth {
            query = query
                .filter(aredl_submissions::submitted_by.eq(user))
                .filter(aredl_submissions::status.eq(SubmissionStatus::Pending));
        }

        let result = query.get_result::<Submission>(conn)?;
        Ok(result)
    }
}

impl SubmissionResolved {
    pub fn from(submission: Submission, db: web::Data<Arc<DbAppState>>, priority: Option<i64>) -> Result<SubmissionResolved, ApiError> {

        let conn = &mut db.connection()?;
        let level = ResolvedLevel::find(db, submission.level_id)?;
        let base_level = BaseLevel {
            id: level.id,
            name: level.name
        };

        let submitter = users::table
            .filter(users::id.eq(submission.submitted_by))
            .select((users::username, users::global_name))
            .first::<(String, String)>(conn)?;
        let submitted_by = BaseUser {
            id: submission.submitted_by,
            username: submitter.0,
            global_name: submitter.1,
        };

        let reviewer: Option<BaseUser> = match submission.reviewer_id {
            Some(reviewer_id) => {
                let reviewer_db = users::table
                    .filter(users::id.eq(reviewer_id))
                    .select((users::username, users::global_name))
                    .first::<(String, String)>(conn)?;
                Some(BaseUser {
                    id: reviewer_id,
                    username: reviewer_db.0,
                    global_name: reviewer_db.1,
                })
            },
            None => None,
        };


        let priority_value = match priority {
            None => {
                aredl_submissions_with_priority::table
                    .filter(aredl_submissions_with_priority::id.eq(submission.id))
                    .select(aredl_submissions_with_priority::priority_value)
                    .first::<i64>(conn)?
            },
            Some(v) => v
        };
        Ok(SubmissionResolved {
            id: submission.id,
            level: base_level,
            submitted_by,
            mobile: submission.mobile,
            ldm_id: submission.ldm_id,
            video_url: submission.video_url,
            raw_url: submission.raw_url,
            status: submission.status,
            reviewer,
            priority: submission.priority,
            is_update: submission.is_update,
            rejection_reason: submission.rejection_reason,
            additional_notes: submission.additional_notes,
            created_at: submission.created_at,
            priority_value,
        })
    }
    pub fn find_one(db: web::Data<Arc<DbAppState>>, id: Uuid, authenticated: Authenticated) -> Result<SubmissionResolved, ApiError> {
        let conn = &mut db.connection()?;
        let has_auth = Authenticated::has_permission(&authenticated, db.clone(), Permission::RecordModify)?;
        
        let mut query = aredl_submissions_with_priority::table
            .filter(aredl_submissions_with_priority::id.eq(id))
            .into_boxed();

        if !has_auth {
            query = query.filter(aredl_submissions_with_priority::submitted_by.eq(authenticated.user_id));
        }

        let submission = query
            .select(SubmissionWithPriority::as_select())
            .first(conn)?;
            
        let level = ResolvedLevel::find(db, submission.level_id)?;
        let base_level = BaseLevel {
            id: level.id,
            name: level.name
        };

        let submitter = users::table
            .filter(users::id.eq(submission.submitted_by))
            .select((users::username, users::global_name))
            .first::<(String, String)>(conn)?;
        let submitted_by = BaseUser {
            id: submission.submitted_by,
            username: submitter.0,
            global_name: submitter.1,
        };

        let reviewer: Option<BaseUser> = match submission.reviewer_id {
            Some(reviewer_id) => {
                let reviewer_db = users::table
                    .filter(users::id.eq(reviewer_id))
                    .select((users::username, users::global_name))
                    .first::<(String, String)>(conn)?;
                Some(BaseUser {
                    id: reviewer_id,
                    username: reviewer_db.0,
                    global_name: reviewer_db.1,
                })
            },
            None => None,
        };

        Ok(SubmissionResolved {
            id: submission.id,
            level: base_level,
            submitted_by,
            mobile: submission.mobile,
            ldm_id: submission.ldm_id,
            video_url: submission.video_url,
            raw_url: submission.raw_url,
            status: submission.status,
            reviewer,
            priority: submission.priority,
            is_update: submission.is_update,
            rejection_reason: submission.rejection_reason,
            additional_notes: submission.additional_notes,
            created_at: submission.created_at,
            priority_value: submission.priority_value,
        })
    }
    pub fn find_highest_priority(db: web::Data<Arc<DbAppState>>, user: Uuid) -> Result<SubmissionResolved, ApiError> {
        let conn = &mut db.connection()?;
        let new_data = SubmissionPatch {
            reviewer_id: Some(user),
            status: Some(SubmissionStatus::Claimed),
            ..Default::default()
        };
        // TODO: maybe this could become one super clean query?
        let highest_priority_id = aredl_submissions_with_priority::table
            .filter(aredl_submissions_with_priority::status.eq(SubmissionStatus::Pending))
            .select((aredl_submissions_with_priority::id, aredl_submissions_with_priority::priority_value))
            .order(aredl_submissions_with_priority::priority_value.desc())
            .limit(1)
            .first::<(Uuid, i64)>(conn)?;
            
        // we don't really need to return the priority value here
        let submission = diesel::update(aredl_submissions::table
            .filter(aredl_submissions::id.eq(highest_priority_id.0)))
            .set(new_data)
            .returning(Submission::as_select())
            .get_result(conn)?;

        let upgraded = SubmissionResolved::from(submission, db, Some(highest_priority_id.1))?;
        
        Ok(upgraded)
    }
}

impl SubmissionQueue {
    pub fn get_queue(db: web::Data<Arc<DbAppState>>) -> Result<Self, ApiError> {
        let conn = &mut db.connection()?;

        let levels = aredl_submissions::table
            .filter(aredl_submissions::status.eq(SubmissionStatus::Pending))
            .count()
            .get_result::<i64>(conn)? as i32;

        Ok(Self { levels_in_queue: levels })
    }
    
}

impl SubmissionPage {
    pub fn find_all<const D: i64>(db: web::Data<Arc<DbAppState>>, page_query: PageQuery<D>, options: SubmissionQueryOptions) -> Result<Paginated<Self>, ApiError> {
        let conn = &mut db.connection()?;
        let query = aredl_submissions::table;



        let total_count: i64 = query
            .count()
            .get_result(conn)?;

        let submissions = query
            .filter(
                options.status_filter.map_or_else(
                    || Box::new(true.into_sql::<Bool>()) as Box<dyn BoxableExpression<_, _, SqlType = Bool>>,
                    |status| Box::new(aredl_submissions::status.eq(status))
                )
            )
            .filter(
                options.mobile_fiter.map_or_else(
                    || Box::new(true.into_sql::<Bool>()) as Box<dyn BoxableExpression<_, _, SqlType = Bool>>,
                    |mobile| Box::new(aredl_submissions::mobile.eq(mobile))
                )
            )
            .filter(
                options.level_filter.map_or_else(
                    || Box::new(true.into_sql::<Bool>()) as Box<dyn BoxableExpression<_, _, SqlType = Bool>>,
                    |level| Box::new(aredl_submissions::level_id.eq(level))
                )
            )
            .filter(
                options.submitter_filter.map_or_else(
                    || Box::new(true.into_sql::<Bool>()) as Box<dyn BoxableExpression<_, _, SqlType = Bool>>,
                    |submitter| Box::new(aredl_submissions::submitted_by.eq(submitter))
                )
            )
            .filter(
                options.priority_filter.map_or_else(
                    || Box::new(true.into_sql::<Bool>()) as Box<dyn BoxableExpression<_, _, SqlType = Bool>>,
                    |priority| Box::new(aredl_submissions::priority.eq(priority))
                )
            )
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select(Submission::as_select())
            .load::<Submission>(conn)?;

        Ok(Paginated::<Self>::from_data(page_query, total_count, Self {
            data: submissions
        }))
    }

    pub fn find_own<const D: i64>(db: web::Data<Arc<DbAppState>>, page_query: PageQuery<D>, options: SubmissionQueryOptions, authenticated: Authenticated) -> Result<Paginated<Self>, ApiError> {
        let conn = &mut db.connection()?;
        let query = aredl_submissions::table;

        let total_count: i64 = query
            .count()
            .get_result(conn)?;

        let submissions = query
            .filter(
                options.status_filter.map_or_else(
                    || Box::new(true.into_sql::<Bool>()) as Box<dyn BoxableExpression<_, _, SqlType = Bool>>,
                    |status| Box::new(aredl_submissions::status.eq(status))
                )
            )
            .filter(
                options.mobile_fiter.map_or_else(
                    || Box::new(true.into_sql::<Bool>()) as Box<dyn BoxableExpression<_, _, SqlType = Bool>>,
                    |mobile| Box::new(aredl_submissions::mobile.eq(mobile))
                )
            )
            .filter(
                options.level_filter.map_or_else(
                    || Box::new(true.into_sql::<Bool>()) as Box<dyn BoxableExpression<_, _, SqlType = Bool>>,
                    |level| Box::new(aredl_submissions::level_id.eq(level))
                )
            )
            .filter(
                options.priority_filter.map_or_else(
                    || Box::new(true.into_sql::<Bool>()) as Box<dyn BoxableExpression<_, _, SqlType = Bool>>,
                    |priority| Box::new(aredl_submissions::priority.eq(priority))
                )
            )
            .filter(aredl_submissions::submitted_by.eq(authenticated.user_id))
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select(Submission::as_select())
            .load::<Submission>(conn)?;

        Ok(Paginated::<Self>::from_data(page_query, total_count, Self {
            data: submissions
        }))
    }
}
