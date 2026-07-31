use crate::{
    app_data::db::DbConnection,
    aredl::levels::id_resolver::level_filter,
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    schema::{
        aredl::{level_notes, levels},
        users,
    },
    users::BaseUser,
};
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg, ExpressionMethods as _, JoinOnDsl as _, QueryDsl as _, RunQueryDsl as _, Selectable,
    SelectableHelper as _,
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::aredl::sql_types::LevelNotesType"]
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
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelNotesResolved {
    pub id: Uuid,
    pub level_id: Uuid,
    pub note: String,
    pub note_type: LevelNotesType,
    pub timestamp: Option<DateTime<Utc>>,
    pub added_by: BaseUser,
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
    pub note: Option<String>,
    pub note_type: Option<LevelNotesType>,
    #[serde(default, with = "double_option")]
    pub timestamp: Option<Option<DateTime<Utc>>>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelNotePost {
    pub note: String,
    pub note_type: LevelNotesType,
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
            .filter(level_notes::level_id.eq_any(level_filter(level_id)?.select(levels::id)))
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
            .inner_join(users::table.on(level_notes::added_by.eq(users::id)))
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
            .values(data)
            .returning(LevelNotes::as_select())
            .get_result(conn)?;

        Ok(notes)
    }

    pub fn update(
        conn: &mut DbConnection,
        data: LevelNoteUpdate,
        id: &Uuid,
    ) -> Result<LevelNotes, ApiError> {
        let notes = diesel::update(level_notes::table)
            .filter(level_notes::id.eq(id))
            .set(data)
            .returning(LevelNotes::as_select())
            .get_result(conn)?;

        Ok(notes)
    }

    pub fn delete(conn: &mut DbConnection, id: &Uuid) -> Result<(), ApiError> {
        diesel::delete(level_notes::table)
            .filter(level_notes::id.eq(id))
            .execute(conn)?;
        Ok(())
    }
}
