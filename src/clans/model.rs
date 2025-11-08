use crate::auth::Authenticated;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{clan_invites, clan_members, clans};
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::{
    ExpressionMethods, OptionalExtension, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::members::ClanMemberAdd;

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=clans, check_for_backend(Pg))]
pub struct Clan {
    /// Internal UUID of the clan.
    pub id: Uuid,
    /// Display name of the clan.
    pub global_name: String,
    /// Short tag of the clan.
    pub tag: String,
    /// Description of the clan.
    pub description: Option<String>,
    /// Timestamp of when the clan was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the clan metadata was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable, ToSchema)]
#[diesel(table_name=clan_members, check_for_backend(Pg))]
pub struct ClanMember {
    /// Internal UUID of the clan member.
    pub id: Uuid,
    /// Internal UUID of the clan.
    pub clan_id: Uuid,
    /// Internal UUID of the user.
    pub user_id: Uuid,
    /// Role of the user in the clan.
    pub role: i32,
    /// Timestamp of when the user joined the clan.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the user was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable, ToSchema)]
#[diesel(table_name=clan_invites, check_for_backend(Pg))]
pub struct ClanInvite {
    /// Internal UUID of the clan invite.
    pub id: Uuid,
    /// Internal UUID of the clan.
    pub clan_id: Uuid,
    /// Internal UUID of the user.
    pub user_id: Uuid,
    /// Internal UUID of the user who invited the user.
    pub invited_by: Uuid,
    /// Timestamp of when the invite was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the invite was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=clans, check_for_backend(Pg))]
pub struct ClanCreate {
    /// Display name of the clan to create.
    pub global_name: String,
    /// Short tag of the clan to create.
    pub tag: String,
    /// Description of the clan to create.
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=clans, check_for_backend(Pg))]
pub struct ClanUpdate {
    /// New display name of the clan.
    pub global_name: Option<String>,
    /// New short tag of the clan.
    pub tag: Option<String>,
    /// New description of the clan.
    pub description: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ClanPage {
    /// List of found clans
    pub data: Vec<Clan>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ClanListQueryOptions {
    pub name_filter: Option<String>,
}

impl Clan {
    pub fn create_empty(conn: &mut DbConnection, clan: ClanCreate) -> Result<Self, ApiError> {
        if clan.global_name.len() > 100 {
            return Err(ApiError::new(
                400,
                "The clan name can at most be 100 characters long.",
            ));
        }

        if clan.tag.len() > 5 {
            return Err(ApiError::new(
                400,
                "The clan tag can at most be 5 characters long.",
            ));
        }

        if clan.description.is_some() && clan.description.as_ref().unwrap().len() > 300 {
            return Err(ApiError::new(
                400,
                "The clan description can at most be 300 characters long.",
            ));
        }

        let clan = diesel::insert_into(clans::table)
            .values(&clan)
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(clan)
    }

    pub fn create_and_join(
        conn: &mut DbConnection,
        clan: ClanCreate,
        authenticated: Authenticated,
    ) -> Result<Self, ApiError> {
        let existing_clan_member = clan_members::table
            .filter(clan_members::user_id.eq(authenticated.user_id))
            .first::<ClanMember>(conn)
            .optional()?;

        if existing_clan_member.is_some() {
            return Err(ApiError::new(400, "You are already in a clan."));
        }

        if clan.global_name.len() > 100 {
            return Err(ApiError::new(
                400,
                "The clan name can at most be 100 characters long.",
            ));
        }

        if clan.tag.len() > 5 {
            return Err(ApiError::new(
                400,
                "The clan tag can at most be 5 characters long.",
            ));
        }

        if clan.description.is_some() && clan.description.as_ref().unwrap().len() > 300 {
            return Err(ApiError::new(
                400,
                "The clan description can at most be 300 characters long.",
            ));
        }

        let clan = diesel::insert_into(clans::table)
            .values(&clan)
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;

        diesel::insert_into(clan_members::table)
            .values(ClanMemberAdd {
                clan_id: clan.id,
                user_id: authenticated.user_id,
            })
            .execute(conn)?;

        diesel::update(clan_members::table)
            .filter(clan_members::clan_id.eq(clan.id))
            .filter(clan_members::user_id.eq(authenticated.user_id))
            .set(clan_members::role.eq(2))
            .execute(conn)?;

        Ok(clan)
    }

    pub fn find<const D: i64>(
        conn: &mut DbConnection,
        options: ClanListQueryOptions,
        page_query: PageQuery<D>,
    ) -> Result<Paginated<ClanPage>, ApiError> {
        let build_query = || {
            let mut q = clans::table.into_boxed::<Pg>();
            if let Some(ref name_like) = options.name_filter {
                q = q.filter(clans::global_name.ilike(name_like));
            }
            q
        };

        let total_count: i64 = build_query().count().get_result(conn)?;

        let entries: Vec<Clan> = build_query()
            .order(clans::global_name.asc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select(Clan::as_select())
            .load(conn)?;

        Ok(Paginated::from_data(
            page_query,
            total_count,
            ClanPage { data: entries },
        ))
    }

    pub fn update(
        conn: &mut DbConnection,
        clan_id: Uuid,
        clan: ClanUpdate,
    ) -> Result<Self, ApiError> {
        if clan.global_name.is_some() && clan.global_name.as_ref().unwrap().len() > 100 {
            return Err(ApiError::new(
                400,
                "The clan name can at most be 100 characters long.",
            ));
        }

        if clan.tag.is_some() && clan.tag.as_ref().unwrap().len() > 5 {
            return Err(ApiError::new(
                400,
                "The clan tag can at most be 5 characters long.",
            ));
        }

        if clan.description.is_some() && clan.description.as_ref().unwrap().len() > 300 {
            return Err(ApiError::new(
                400,
                "The clan description can at most be 300 characters long.",
            ));
        }

        let updated_clan = diesel::update(clans::table.filter(clans::id.eq(clan_id)))
            .set(&clan)
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(updated_clan)
    }

    pub fn delete(conn: &mut DbConnection, clan_id: Uuid) -> Result<Clan, ApiError> {
        let clan = diesel::delete(clans::table.filter(clans::id.eq(clan_id)))
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(clan)
    }
}
