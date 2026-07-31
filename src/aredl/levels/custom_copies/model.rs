use crate::{
    app_data::db::DbConnection,
    aredl::levels::id_resolver::level_filter,
    auth::Authenticated,
    error_handler::ApiError,
    schema::{
        aredl::{level_custom_copies, levels},
        users,
    },
    users::BaseUser,
};
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg, ExpressionMethods as _, JoinOnDsl as _, PgTextExpressionMethods as _, QueryDsl as _,
    RunQueryDsl as _, Selectable, SelectableHelper as _,
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::aredl::sql_types::CustomIdStatus"]
#[DbValueStyle = "PascalCase"]
pub enum LevelCustomCopyStatus {
    /// This ID is suggested for use and officially displayed on the site.
    Published,
    /// This ID is not the one suggested for use, but is allowed in records
    Allowed,
    /// This ID cannot be used in records
    Banned,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::aredl::sql_types::CustomIdType"]
#[DbValueStyle = "PascalCase"]
pub enum LevelCustomCopyType {
    /// This level fixes a bug in the offical level
    Bugfix,
    /// This level is made for use with Globed Deathlink
    GlobedCopy,
    /// This level is a Low Detail Mode of the official level
    Ldm,
    Other,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, ToSchema)]
#[diesel(table_name = level_custom_copies, check_for_backend(Pg))]
pub struct LevelCustomCopy {
    /// The internal ID of this custom copy entry
    pub id: Uuid,
    /// The internal ID of the level this custom copy is for
    pub level_id: Uuid,
    /// The in-game ID of this copy
    pub copy_id: i32,
    /// The moderator who added this custom copy
    pub added_by: Uuid,
    /// The type of this custom copy
    pub id_type: LevelCustomCopyType,
    /// The status of this custom copy on the site
    pub status: LevelCustomCopyStatus,
    /// The description of what this custom copy changes, if any
    pub description: Option<String>,
    /// The time this custom copy was added
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelCustomCopyResolved {
    pub id: Uuid,
    pub level_id: Uuid,
    pub copy_id: i32,
    pub added_by: BaseUser,
    pub id_type: LevelCustomCopyType,
    pub description: Option<String>,
    pub status: LevelCustomCopyStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = level_custom_copies)]
pub struct LevelCustomCopyInsert {
    pub level_id: Uuid,
    pub copy_id: i32,
    pub id_type: LevelCustomCopyType,
    pub added_by: Uuid,
    pub status: LevelCustomCopyStatus,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = level_custom_copies, check_for_backend(Pg))]
pub struct LevelCustomCopyUpdate {
    pub copy_id: Option<i32>,
    pub id_type: Option<LevelCustomCopyType>,
    pub status: Option<LevelCustomCopyStatus>,
    #[serde(default, with = "double_option")]
    pub description: Option<Option<String>>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelCustomCopyBody {
    pub copy_id: i32,
    pub id_type: LevelCustomCopyType,
    pub status: LevelCustomCopyStatus,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelCustomCopyQueryOptions {
    pub type_filter: Option<LevelCustomCopyType>,
    pub status_filter: Option<LevelCustomCopyStatus>,
    pub description_filter: Option<Option<String>>,
    pub added_by: Option<Uuid>,
}

impl LevelCustomCopy {
    pub fn find_all_level(
        conn: &mut DbConnection,
        filters: &LevelCustomCopyQueryOptions,
        level_id: &str,
    ) -> Result<Vec<LevelCustomCopyResolved>, ApiError> {
        let mut query = level_custom_copies::table
            .filter(
                level_custom_copies::level_id.eq_any(level_filter(level_id)?.select(levels::id)),
            )
            .into_boxed::<Pg>();

        if let Some(added_by) = filters.added_by {
            query = query.filter(level_custom_copies::added_by.eq(added_by));
        }
        if let Some(custom_copy_type) = filters.type_filter.as_ref() {
            query = query.filter(level_custom_copies::id_type.eq(custom_copy_type));
        }
        if let Some(status) = filters.status_filter.as_ref() {
            query = query.filter(level_custom_copies::status.eq(status));
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
            .inner_join(users::table.on(level_custom_copies::added_by.eq(users::id)))
            .select((LevelCustomCopy::as_select(), BaseUser::as_select()))
            .load::<(LevelCustomCopy, BaseUser)>(conn)?
            .into_iter()
            .map(|(custom_copy, moderator)| LevelCustomCopyResolved {
                id: custom_copy.id,
                level_id: custom_copy.level_id,
                copy_id: custom_copy.copy_id,
                added_by: moderator,
                id_type: custom_copy.id_type,
                description: custom_copy.description,
                status: custom_copy.status,
                created_at: custom_copy.created_at,
            })
            .collect::<Vec<LevelCustomCopyResolved>>();

        Ok(custom_copies)
    }

    pub fn create(
        conn: &mut DbConnection,
        body: LevelCustomCopyBody,
        level_id: Uuid,
        auth: &Authenticated,
    ) -> Result<LevelCustomCopy, ApiError> {
        let data = LevelCustomCopyInsert {
            level_id,
            copy_id: body.copy_id,
            id_type: body.id_type,
            status: body.status,
            description: body.description,
            added_by: auth.user_id,
        };
        let custom_copy = diesel::insert_into(level_custom_copies::table)
            .values(data)
            .returning(LevelCustomCopy::as_select())
            .get_result(conn)?;

        Ok(custom_copy)
    }

    pub fn update(
        conn: &mut DbConnection,
        data: LevelCustomCopyUpdate,
        id: &Uuid,
    ) -> Result<LevelCustomCopy, ApiError> {
        let custom_copy = diesel::update(level_custom_copies::table)
            .filter(level_custom_copies::id.eq(id))
            .set(data)
            .returning(LevelCustomCopy::as_select())
            .get_result(conn)?;

        Ok(custom_copy)
    }

    pub fn delete(conn: &mut DbConnection, id: &Uuid) -> Result<(), ApiError> {
        diesel::delete(level_custom_copies::table)
            .filter(level_custom_copies::id.eq(id))
            .execute(conn)?;

        Ok(())
    }
}
