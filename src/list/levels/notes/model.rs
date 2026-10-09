use crate::list::List;
use crate::{
    app_data::db::DbConnection,
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    list::levels::id_resolver::level_filter,
    schema::{level_notes, levels, users},
    users::BaseUser,
};
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, Selectable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::LevelNotesType"]
#[DbValueStyle = "PascalCase"]
pub enum LevelNotesType {
    ReviewerNotes,
    PublicNotes,
    Other,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, ToSchema)]
#[diesel(table_name = level_notes, check_for_backend(Pg))]
pub struct LevelNotes {
    /// The internal ID of this note
    pub id: Uuid,
    /// The internal ID of the level this note is for
    pub level_id: Uuid,
    /// The content of this note
    pub note: String,
    /// The type of this note.
    pub note_type: LevelNotesType,
    /// An optional timestamp after which this note should apply
    pub timestamp: Option<DateTime<Utc>>,
    /// The moderator who added this note
    pub added_by: Uuid,
    /// The timestamp when this note was added
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelNotesResolved {
    /// The internal ID of this note
    pub id: Uuid,
    /// The internal ID of the level this note is for
    pub level_id: Uuid,
    /// The content of this note
    pub note: String,
    /// The type of this note.
    pub note_type: LevelNotesType,
    /// An optional timestamp after which this note should apply
    pub timestamp: Option<DateTime<Utc>>,
    /// The moderator who added this note
    pub added_by: BaseUser,
    /// The timestamp when this note was created
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = level_notes)]
pub struct LevelNoteInsert {
    pub level_id: Uuid,
    pub note: String,
    pub note_type: LevelNotesType,
    pub timestamp: Option<DateTime<Utc>>,
    pub added_by: Uuid,
}

#[derive(Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = level_notes, check_for_backend(Pg))]
pub struct LevelNoteUpdate {
    /// The content of this note
    pub note: Option<String>,
    /// The type of this note.
    pub note_type: Option<LevelNotesType>,
    /// An optional timestamp after which this note should apply
    #[serde(default, with = "double_option")]
    pub timestamp: Option<Option<DateTime<Utc>>>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelNotePost {
    /// The content of this note
    pub note: String,
    /// The type of this note.
    pub note_type: LevelNotesType,
    /// An optional timestamp after which this note should apply
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelNotesQueryOptions {
    pub type_filter: Option<LevelNotesType>,
    pub added_by: Option<Uuid>,
}

impl LevelNotes {
    pub fn find_all_level(
        conn: &mut DbConnection,
        filters: &LevelNotesQueryOptions,
        list: List,
        level_id: &str,
        authenticated: Option<Authenticated>,
    ) -> Result<Vec<LevelNotesResolved>, ApiError> {
        let is_reviewer = match authenticated {
            Some(authenticated) => {
                authenticated.has_permission(conn, Permission::SubmissionReview)?
            }
            None => false,
        };

        let mut query = level_notes::table
            .filter(level_notes::list_id.eq(list))
            .filter(level_notes::level_id.eq_any(level_filter(list, level_id)?.select(levels::id)))
            .into_boxed::<Pg>();
        if let Some(user_id) = filters.added_by {
            query = query.filter(level_notes::added_by.eq(user_id));
        }
        if let Some(note_type) = filters.type_filter.as_ref() {
            query = query.filter(level_notes::note_type.eq(note_type));
        }
        if !is_reviewer {
            query = query.filter(level_notes::note_type.ne(LevelNotesType::ReviewerNotes));
        }

        let notes = query
            .order(level_notes::created_at.desc())
            .inner_join(users::table)
            .select((LevelNotes::as_select(), BaseUser::as_select()))
            .load(conn)?
            .into_iter()
            .map(|(note, moderator)| LevelNotesResolved {
                id: note.id,
                level_id: note.level_id,
                note: note.note,
                added_by: moderator,
                note_type: note.note_type,
                timestamp: note.timestamp,
                created_at: note.created_at,
            })
            .collect::<Vec<LevelNotesResolved>>();

        Ok(notes)
    }

    pub fn create(
        conn: &mut DbConnection,
        body: LevelNotePost,
        list: List,
        level_id: Uuid,
        auth: &Authenticated,
    ) -> Result<LevelNotes, ApiError> {
        let data = LevelNoteInsert {
            level_id,
            note: body.note,
            note_type: body.note_type,
            timestamp: body.timestamp,
            added_by: auth.user_id,
        };
        let notes = diesel::insert_into(level_notes::table)
            .values((level_notes::list_id.eq(list), data))
            .returning(LevelNotes::as_select())
            .get_result(conn)?;

        Ok(notes)
    }

    pub fn update(
        conn: &mut DbConnection,
        data: LevelNoteUpdate,
        list: List,
        level_id: Uuid,
        id: &Uuid,
    ) -> Result<LevelNotes, ApiError> {
        let notes = diesel::update(level_notes::table)
            .filter(level_notes::list_id.eq(list))
            .filter(level_notes::level_id.eq(level_id))
            .filter(level_notes::id.eq(id))
            .set(data)
            .returning(LevelNotes::as_select())
            .get_result(conn)?;

        Ok(notes)
    }

    pub fn delete(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        id: &Uuid,
    ) -> Result<(), ApiError> {
        diesel::delete(level_notes::table)
            .filter(level_notes::list_id.eq(list))
            .filter(level_notes::level_id.eq(level_id))
            .filter(level_notes::id.eq(id))
            .execute(conn)?;
        Ok(())
    }
}
