use crate::{
    app_data::db::DbConnection,
    audit::{AuditEntityType, Auditable},
    auth::Authenticated,
    error_handler::ApiError,
    list::{levels::id_resolver::level_filter, List},
    schema::{level_custom_copies, levels, users},
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
#[ExistingTypePath = "crate::schema::sql_types::CustomIdStatus"]
#[DbValueStyle = "PascalCase"]
pub enum LevelCustomCopyStatus {
    /// This ID is allowed in records
    Allowed,
    /// This ID cannot be used in records
    Banned,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::CustomIdType"]
#[DbValueStyle = "PascalCase"]
pub enum LevelCustomCopyType {
    /// This level fixes a bug in the offical level
    BugFix,
    /// This level is made for use with Globed Deathlink
    GlobedCopy,
    /// This level is a Low Detail Mode of the official level
    Ldm,
    /// This level contains start positions
    StartPos,
    Other,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name = level_custom_copies, check_for_backend(Pg))]
pub struct LevelCustomCopy {
    /// The list this custom copy belongs to.
    pub list_id: List,
    /// The internal ID of this custom copy entry
    pub id: Uuid,
    /// The internal ID this custom copy is for
    pub level_id: Uuid,
    /// The in-game ID of this custom copy
    pub copy_id: i32,
    /// The moderator who added this custom copy
    pub added_by: Uuid,
    /// The types of this custom copy
    #[schema(min_items = 1)]
    pub id_types: Vec<LevelCustomCopyType>,
    /// The status of this custom copy on the site
    pub status: LevelCustomCopyStatus,
    /// Whether this custom copy should be displayed on the site or hidden
    pub published: bool,
    /// The description of what this custom copy changes, if any
    pub description: Option<String>,
    /// The time this custom copy was added
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelCustomCopyResolved {
    /// The internal ID of this custom copy entry
    pub id: Uuid,
    /// The internal ID of the level this custom copy is for
    pub level_id: Uuid,
    /// The in-game ID of this copy
    pub copy_id: i32,
    /// The types of this custom copy
    #[schema(min_items = 1)]
    pub id_types: Vec<LevelCustomCopyType>,
    /// The status of this custom copy on the site
    pub status: LevelCustomCopyStatus,
    /// Whether this custom copy should be displayed on the site or hidden
    pub published: bool,
    /// The moderator who added this custom copy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_by: Option<BaseUser>,
    /// The description of what this custom copy changes, if any
    pub description: Option<String>,
    /// The time this custom copy was added
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = level_custom_copies)]
pub struct LevelCustomCopyInsert {
    pub level_id: Uuid,
    pub copy_id: i32,
    pub id_types: Vec<LevelCustomCopyType>,
    pub status: LevelCustomCopyStatus,
    pub published: bool,
    pub description: Option<String>,
    pub added_by: Uuid,
}

#[derive(Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = level_custom_copies, check_for_backend(Pg))]
pub struct LevelCustomCopyUpdate {
    /// The in-game ID of this copy
    pub copy_id: Option<i32>,
    /// The types of this custom copy
    #[schema(min_items = 1)]
    pub id_types: Option<Vec<LevelCustomCopyType>>,
    /// The status of this custom copy on the site
    pub status: Option<LevelCustomCopyStatus>,
    /// Whether this custom copy should be displayed on the site or hidden
    pub published: Option<bool>,
    /// The description of what this custom copy changes, if any
    #[serde(default, with = "double_option")]
    pub description: Option<Option<String>>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelCustomCopyBody {
    /// The in-game ID of this copy
    pub copy_id: i32,
    /// The types of this custom copy
    #[schema(min_items = 1)]
    pub id_types: Vec<LevelCustomCopyType>,
    /// The status of this custom copy on the site
    pub status: LevelCustomCopyStatus,
    /// Whether this custom copy should be displayed on the site or hidden
    #[serde(default)]
    #[schema(default = false)]
    pub published: bool,
    /// The description of what this custom copy changes, if any
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Default)]
pub struct LevelCustomCopyQueryOptions {
    pub type_filter: Option<LevelCustomCopyType>,
    pub status_filter: Option<LevelCustomCopyStatus>,
    pub published_filter: Option<bool>,
    pub description_filter: Option<Option<String>>,
    pub added_by: Option<Uuid>,
}

impl Auditable for LevelCustomCopy {
    const ENTITY_TYPE: AuditEntityType = AuditEntityType::LevelCustomCopy;

    fn entity_id(&self) -> Uuid {
        self.id
    }

    fn parent_entity_id(&self) -> Option<Uuid> {
        Some(self.level_id)
    }

    fn entity_display(&self, _conn: &mut DbConnection) -> Result<String, ApiError> {
        Ok(format!("{} ({:?})", self.copy_id, self.id_types))
    }

    fn parent_display(&self, conn: &mut DbConnection) -> Result<Option<String>, ApiError> {
        Ok(Some(
            levels::table
                .filter(levels::list_id.eq(self.list_id))
                .find(self.level_id)
                .select(levels::name)
                .first(conn)?,
        ))
    }

    fn list_id(&self) -> Option<List> {
        Some(self.list_id)
    }
}

impl LevelCustomCopy {
    pub fn find_all(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
    ) -> Result<Vec<Self>, ApiError> {
        Ok(level_custom_copies::table
            .filter(level_custom_copies::list_id.eq(list))
            .filter(level_custom_copies::level_id.eq(level_id))
            .order(level_custom_copies::created_at.desc())
            .select(Self::as_select())
            .load(conn)?)
    }

    pub fn find_for_update(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        id: Uuid,
    ) -> Result<Self, ApiError> {
        Ok(level_custom_copies::table
            .filter(level_custom_copies::list_id.eq(list))
            .filter(level_custom_copies::level_id.eq(level_id))
            .find(id)
            .for_update()
            .select(Self::as_select())
            .first(conn)?)
    }

    pub fn create(
        conn: &mut DbConnection,
        body: LevelCustomCopyBody,
        list: List,
        level_id: Uuid,
        auth: &Authenticated,
    ) -> Result<Self, ApiError> {
        let data = LevelCustomCopyInsert {
            level_id,
            status: body.status,
            published: body.published,
            id_types: body.id_types,
            copy_id: body.copy_id,
            description: body.description,
            added_by: auth.user_id,
        };
        let custom_copy = diesel::insert_into(level_custom_copies::table)
            .values((level_custom_copies::list_id.eq(list), data))
            .returning(Self::as_select())
            .get_result(conn)?;

        Ok(custom_copy)
    }
    pub fn update(
        conn: &mut DbConnection,
        data: LevelCustomCopyUpdate,
        list: List,
        level_id: Uuid,
        id: &Uuid,
    ) -> Result<Self, ApiError> {
        let custom_copy = diesel::update(level_custom_copies::table)
            .filter(level_custom_copies::list_id.eq(list))
            .filter(level_custom_copies::level_id.eq(level_id))
            .filter(level_custom_copies::id.eq(id))
            .set(data)
            .returning(Self::as_select())
            .get_result(conn)?;

        Ok(custom_copy)
    }

    pub fn delete(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        id: &Uuid,
    ) -> Result<Option<Self>, ApiError> {
        let deleted = diesel::delete(level_custom_copies::table)
            .filter(level_custom_copies::list_id.eq(list))
            .filter(level_custom_copies::level_id.eq(level_id))
            .filter(level_custom_copies::id.eq(id))
            .returning(Self::as_select())
            .get_result(conn)
            .optional()?;

        Ok(deleted)
    }
}

impl LevelCustomCopyResolved {
    pub fn find_all(
        conn: &mut DbConnection,
        filters: &LevelCustomCopyQueryOptions,
        list: List,
        level_id: &str,
    ) -> Result<Vec<Self>, ApiError> {
        let mut query = level_custom_copies::table
            .filter(level_custom_copies::list_id.eq(list))
            .filter(
                level_custom_copies::level_id
                    .eq_any(level_filter(list, level_id)?.select(levels::id)),
            )
            .into_boxed::<Pg>();

        if let Some(added_by) = filters.added_by {
            query = query.filter(level_custom_copies::added_by.eq(added_by));
        }
        if let Some(custom_copy_type) = filters.type_filter.as_ref() {
            query = query.filter(level_custom_copies::id_types.contains(vec![custom_copy_type]));
        }
        if let Some(status) = filters.status_filter.as_ref() {
            query = query.filter(level_custom_copies::status.eq(status));
        }
        if let Some(published) = filters.published_filter {
            query = query.filter(level_custom_copies::published.eq(published));
        }
        if let Some(description_filter) = filters.description_filter.as_ref() {
            match description_filter {
                Some(description) => {
                    query = query.filter(level_custom_copies::description.ilike(description));
                }
                None => query = query.filter(level_custom_copies::description.is_null()),
            }
        }

        let custom_copies = query
            .order(level_custom_copies::created_at.desc())
            .inner_join(users::table)
            .select((LevelCustomCopy::as_select(), BaseUser::as_select()))
            .load::<(LevelCustomCopy, BaseUser)>(conn)?
            .into_iter()
            .map(|(custom_copy, moderator)| Self {
                id: custom_copy.id,
                level_id: custom_copy.level_id,
                copy_id: custom_copy.copy_id,
                id_types: custom_copy.id_types,
                status: custom_copy.status,
                published: custom_copy.published,
                added_by: Some(moderator),
                description: custom_copy.description,
                created_at: custom_copy.created_at,
            })
            .collect::<Vec<Self>>();

        Ok(custom_copies)
    }
}
