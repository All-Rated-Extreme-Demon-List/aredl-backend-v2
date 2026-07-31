use crate::{
    app_data::db::DbConnection,
    aredl::levels::id_resolver::level_filter,
    error_handler::ApiError,
    schema::aredl::{level_updates, levels},
};
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg, ExpressionMethods as _, QueryDsl as _, RunQueryDsl as _, Selectable,
    SelectableHelper as _,
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::aredl::sql_types::LevelUpdateType"]
#[DbValueStyle = "PascalCase"]
pub enum LevelUpdateType {
    Buff,
    Nerf,
    Balance,
    BugFix,
    Other,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, ToSchema)]
#[diesel(table_name = level_updates, check_for_backend(Pg))]
pub struct LevelUpdateEntry {
    /// The internal ID of this update
    pub id: Uuid,
    /// The internal ID of the level this update is for
    pub level_id: Uuid,
    /// Optional changelog text for this update
    pub changelog: Option<String>,
    /// The type of this update
    pub update_type: LevelUpdateType,
    /// When this update applies
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = level_updates)]
pub struct LevelUpdateEntryInsert {
    pub level_id: Uuid,
    pub changelog: Option<String>,
    pub update_type: LevelUpdateType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = level_updates, check_for_backend(Pg))]
pub struct LevelUpdateEntryUpdate {
    #[serde(default, with = "double_option")]
    pub changelog: Option<Option<String>>,
    pub update_type: Option<LevelUpdateType>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelUpdateEntryPost {
    pub changelog: Option<String>,
    pub update_type: LevelUpdateType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelUpdateEntryQueryOptions {
    pub type_filter: Option<LevelUpdateType>,
}

impl LevelUpdateEntry {
    pub fn find_all_level(
        conn: &mut DbConnection,
        filters: &LevelUpdateEntryQueryOptions,
        level_id: &str,
    ) -> Result<Vec<LevelUpdateEntry>, ApiError> {
        let mut query = level_updates::table
            .filter(level_updates::level_id.eq_any(level_filter(level_id)?.select(levels::id)))
            .into_boxed::<Pg>();

        if let Some(update_type) = filters.type_filter.as_ref() {
            query = query.filter(level_updates::update_type.eq(update_type));
        }

        let updates = query
            .order((
                level_updates::timestamp.desc(),
                level_updates::created_at.desc(),
            ))
            .select(LevelUpdateEntry::as_select())
            .load(conn)?;

        Ok(updates)
    }

    pub fn create(
        conn: &mut DbConnection,
        body: LevelUpdateEntryPost,
        level_id: Uuid,
    ) -> Result<Self, ApiError> {
        let data = LevelUpdateEntryInsert {
            level_id,
            changelog: body.changelog,
            update_type: body.update_type,
            timestamp: body.timestamp,
        };
        let update = diesel::insert_into(level_updates::table)
            .values(data)
            .returning(Self::as_select())
            .get_result(conn)?;

        Ok(update)
    }

    pub fn update(
        conn: &mut DbConnection,
        data: LevelUpdateEntryUpdate,
        id: &Uuid,
    ) -> Result<Self, ApiError> {
        let update = diesel::update(level_updates::table)
            .filter(level_updates::id.eq(id))
            .set(data)
            .returning(Self::as_select())
            .get_result(conn)?;

        Ok(update)
    }

    pub fn delete(conn: &mut DbConnection, id: &Uuid) -> Result<(), ApiError> {
        diesel::delete(level_updates::table)
            .filter(level_updates::id.eq(id))
            .execute(conn)?;
        Ok(())
    }
}
