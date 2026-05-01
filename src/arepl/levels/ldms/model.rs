use crate::{
    app_data::db::DbConnection,
    arepl::levels::id_resolver::level_filter,
    auth::Authenticated,
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
    schema::{
        arepl::{level_ldms, levels},
        users,
    },
    users::BaseUser,
};
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg, ExpressionMethods, JoinOnDsl, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
    Selectable, SelectableHelper,
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::arepl::sql_types::CustomIdStatus"]
#[DbValueStyle = "PascalCase"]
pub enum LevelLDMStatus {
    /// This ID is the one suggested for use. Levels can only have 1
    /// "Published" ID per type per level (e.g. one bugfix, one globed copy, etc.)
    Published,
    /// This ID is not the one suggested for use, but is allowed in records
    Allowed,
    /// This ID cannot be used in records
    Banned,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::arepl::sql_types::CustomIdType"]
#[DbValueStyle = "PascalCase"]
pub enum LevelLDMType {
    /// This level fixes a bug in the offical level
    Bugfix,
    /// This level is made for use with Globed Deathlink
    GlobedCopy,
    /// This level is a Low Detail Mode of the official level
    Ldm,
    Other,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name = level_ldms, check_for_backend(Pg))]
pub struct LevelLDM {
    /// The internal ID of this LDM entry
    pub id: Uuid,
    /// The internal ID this LDM is for
    pub level_id: Uuid,
    /// The in-game ID of this LDM
    pub ldm_id: i32,
    /// The moderator who added this LDM
    pub added_by: Uuid,
    pub id_type: LevelLDMType,
    pub status: LevelLDMStatus,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelLDMResolved {
    pub id: Uuid,
    pub level_id: Uuid,
    pub ldm_id: i32,
    pub id_type: LevelLDMType,
    pub status: LevelLDMStatus,
    pub added_by: BaseUser,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelLDMResolvedPage {
    pub data: Vec<LevelLDMResolved>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = level_ldms)]
pub struct LevelLDMInsert {
    pub level_id: Uuid,
    pub ldm_id: i32,
    pub id_type: LevelLDMType,
    pub status: LevelLDMStatus,
    pub description: Option<String>,
    pub added_by: Uuid,
}

#[derive(Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = level_ldms, check_for_backend(Pg))]
pub struct LevelLDMUpdate {
    pub ldm_id: Option<i32>,
    pub id_type: Option<LevelLDMType>,
    pub status: Option<LevelLDMStatus>,
    pub description: Option<Option<String>>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelLDMBody {
    pub ldm_id: i32,
    pub id_type: LevelLDMType,
    pub status: LevelLDMStatus,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelLDMQueryOptions {
    pub level_id: Option<String>,
    pub type_filter: Option<LevelLDMType>,
    pub status_filter: Option<LevelLDMStatus>,
    pub description_filter: Option<Option<String>>,
    pub added_by: Option<Uuid>,
}

impl LevelLDM {
    pub fn find_all<const D: i64>(
        conn: &mut DbConnection,
        filters: LevelLDMQueryOptions,
        page_query: PageQuery<D>,
    ) -> Result<Paginated<LevelLDMResolvedPage>, ApiError> {
        let build_filtered = || -> Result<_, ApiError> {
            let mut query = level_ldms::table.into_boxed::<Pg>();

            if let Some(level_id) = filters.level_id.as_deref() {
                query = query.filter(
                    level_ldms::level_id.eq_any(level_filter(level_id)?.select(levels::id)),
                );
            }
            if let Some(added_by) = filters.added_by {
                query = query.filter(level_ldms::added_by.eq(added_by));
            }
            if let Some(ldm_type) = filters.type_filter.as_ref() {
                query = query.filter(level_ldms::id_type.eq(ldm_type));
            }
            if let Some(status) = filters.status_filter.as_ref() {
                query = query.filter(level_ldms::status.eq(status));
            }
            if let Some(description_filter) = filters.description_filter.as_ref() {
                match description_filter {
                    Some(description) => {
                        query = query.filter(level_ldms::description.ilike(description));
                    }
                    None => query = query.filter(level_ldms::description.is_null()),
                }
            }

            Ok(query)
        };

        let count = build_filtered()?.count().get_result::<i64>(conn)?;

        let query = build_filtered()?
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .order(level_ldms::created_at.desc())
            .inner_join(users::table.on(level_ldms::added_by.eq(users::id)))
            .select((LevelLDM::as_select(), BaseUser::as_select()));

        let ldms: Vec<(LevelLDM, BaseUser)> = query.load(conn)?;

        let ldms = ldms
            .into_iter()
            .map(|(ldm, moderator)| LevelLDMResolved {
                id: ldm.id,
                level_id: ldm.level_id,
                ldm_id: ldm.ldm_id,
                id_type: ldm.id_type,
                status: ldm.status,
                added_by: moderator,
                description: ldm.description,
                created_at: ldm.created_at,
            })
            .collect::<Vec<LevelLDMResolved>>();

        Ok(Paginated::from_data(
            page_query,
            count,
            LevelLDMResolvedPage { data: ldms },
        ))
    }

    pub fn create(
        conn: &mut DbConnection,
        body: LevelLDMBody,
        level_id: Uuid,
        auth: Authenticated,
    ) -> Result<LevelLDM, ApiError> {
        let data = LevelLDMInsert {
            level_id,
            status: body.status,
            id_type: body.id_type,
            ldm_id: body.ldm_id,
            description: body.description,
            added_by: auth.user_id,
        };
        let ldm = diesel::insert_into(level_ldms::table)
            .values(data)
            .returning(LevelLDM::as_select())
            .get_result(conn)?;

        Ok(ldm)
    }
    pub fn update(
        conn: &mut DbConnection,
        data: LevelLDMUpdate,
        id: Uuid,
    ) -> Result<LevelLDM, ApiError> {
        let ldm = diesel::update(level_ldms::table)
            .filter(level_ldms::id.eq(id))
            .set(data)
            .returning(LevelLDM::as_select())
            .get_result(conn)?;

        Ok(ldm)
    }
    pub fn delete(conn: &mut DbConnection, id: Uuid) -> Result<(), ApiError> {
        diesel::delete(level_ldms::table)
            .filter(level_ldms::id.eq(id))
            .execute(conn)?;

        Ok(())
    }
}
