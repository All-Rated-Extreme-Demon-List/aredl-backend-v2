use crate::app_data::db::DbConnection;
use crate::audit::{AuditEntityType, Auditable};
use crate::error_handler::ApiError;
use crate::list::List;
use crate::schema::{level_tags, tags};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, DbEnum, ToSchema)]
#[ExistingTypePath = "crate::schema::sql_types::TagCategory"]
#[DbValueStyle = "PascalCase"]
pub enum TagCategory {
    Version,
    Length,
    Gamemode,
    Gameplay,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, ToSchema)]
#[diesel(table_name = tags)]
pub struct Tag {
    /// Internal tag UUID.
    pub id: Uuid,
    /// The list this tag applies to.
    pub list_id: List,
    /// Name of the tag (must be unique for the list)
    pub name: String,
    /// Description of the tag.
    pub description: String,
    /// Category of the tag.
    pub category: TagCategory,
}

#[derive(Debug, Serialize, Deserialize, Insertable, ToSchema)]
#[diesel(table_name = tags)]
pub struct TagCreate {
    /// Name of the tag (must be unique for the list)
    pub name: String,
    /// Description of the tag.
    pub description: String,
    /// Category of the tag.
    pub category: TagCategory,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = tags)]
pub struct TagUpdate {
    /// Name of the tag (must be unique for the list)
    pub name: Option<String>,
    /// Description of the tag.
    pub description: Option<String>,
    /// Category of the tag.
    pub category: Option<TagCategory>,
}

impl Tag {
    pub fn find_all_level(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
    ) -> Result<Vec<Self>, ApiError> {
        Ok(tags::table
            .inner_join(
                level_tags::table.on(level_tags::tag_id
                    .eq(tags::id)
                    .and(level_tags::list_id.eq(tags::list_id))),
            )
            .filter(level_tags::list_id.eq(list))
            .filter(level_tags::level_id.eq(level_id))
            .order((tags::category, tags::name, tags::id))
            .select(Self::as_select())
            .load(conn)?)
    }

    pub fn find_all(conn: &mut DbConnection, list: List) -> Result<Vec<Self>, ApiError> {
        Ok(tags::table
            .filter(tags::list_id.eq(list))
            .order((tags::category, tags::name, tags::id))
            .select(Self::as_select())
            .load(conn)?)
    }

    pub fn find_for_update(
        conn: &mut DbConnection,
        list: List,
        id: Uuid,
    ) -> Result<Self, ApiError> {
        Ok(tags::table
            .filter(tags::list_id.eq(list))
            .find(id)
            .for_update()
            .select(Self::as_select())
            .first(conn)?)
    }

    pub fn create(conn: &mut DbConnection, list: List, tag: TagCreate) -> Result<Self, ApiError> {
        Ok(diesel::insert_into(tags::table)
            .values((tags::list_id.eq(list), tag))
            .returning(Self::as_select())
            .get_result(conn)?)
    }

    pub fn update(
        conn: &mut DbConnection,
        list: List,
        id: Uuid,
        tag: TagUpdate,
    ) -> Result<Self, ApiError> {
        Ok(
            diesel::update(tags::table.filter(tags::list_id.eq(list)).find(id))
                .set((tag, tags::id.eq(id)))
                .returning(Self::as_select())
                .get_result(conn)?,
        )
    }

    pub fn delete(conn: &mut DbConnection, list: List, id: Uuid) -> Result<Self, ApiError> {
        Ok(
            diesel::delete(tags::table.filter(tags::list_id.eq(list)).find(id))
                .returning(Self::as_select())
                .get_result(conn)?,
        )
    }
}

impl Auditable for Tag {
    const ENTITY_TYPE: AuditEntityType = AuditEntityType::Tag;

    fn entity_id(&self) -> Uuid {
        self.id
    }
    fn list_id(&self) -> Option<List> {
        Some(self.list_id)
    }
    fn entity_display(&self, _conn: &mut DbConnection) -> Result<String, ApiError> {
        Ok(self.name.clone())
    }
}
