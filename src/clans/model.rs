use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::{BoxableExpression, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use diesel::expression::AsExpression;
use diesel::sql_types::Bool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{clans, clan_members};

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
	pub created_at: NaiveDateTime,
    /// Timestamp of when the clan metadata was last updated.
	pub updated_at: NaiveDateTime
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
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=clans, check_for_backend(Pg))]
pub struct ClanCreate {
    /// Display name of the clan to create.
	pub global_name: String,
    /// Short tag of the clan to create.
	pub tag: String,
    /// Description of the clan to create.
	pub description: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=clans, check_for_backend(Pg))]
pub struct ClanUpdate {
    /// New display name of the clan.
	pub global_name: Option<String>,
    /// New short tag of the clan.
	pub tag: Option<String>,
    /// New description of the clan.
	pub description: Option<String>
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ClanPage {
    /// List of found clans
    pub data: Vec<Clan>
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ClanListQueryOptions {
    pub name_filter: Option<String>
}

impl Clan {
    pub fn create(conn: &mut DbConnection, clan: ClanCreate) -> Result<Self, ApiError> {
        let clan = diesel::insert_into(clans::table)
            .values(&clan)
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(clan)
    }

    pub fn find<const D: i64>(
        conn: &mut DbConnection,
		options: ClanListQueryOptions,
        page_query: PageQuery<D>)
        -> Result<Paginated<ClanPage>, ApiError>
    {
        let name_filter: Box<dyn BoxableExpression<_, _, SqlType = Bool>> =
            match options.name_filter.clone() {
                Some(filter) => Box::new(clans::global_name.ilike(filter)),
                None => Box::new(<bool as AsExpression<Bool>>::as_expression(true)),
            };

        let entries =
            clans::table
                .filter(name_filter)
                .order(clans::global_name)
                .limit(page_query.per_page())
                .offset(page_query.offset())
                .select(Clan::as_select())
                .load::<Clan>(conn)?;
		
		let name_filter: Box<dyn BoxableExpression<_, _, SqlType = Bool>> =
			match options.name_filter.clone() {
				Some(filter) => Box::new(clans::global_name.ilike(filter)),
				None => Box::new(<bool as AsExpression<Bool>>::as_expression(true)),
			};

        let count = clans::table
            .filter(name_filter)
            .count()
            .get_result(conn)?;

        Ok(Paginated::<ClanPage>::from_data(page_query, count, ClanPage {
            data: entries
        }))
    }

    pub fn update(
        conn: &mut DbConnection,
        clan_id: Uuid,
        clan: ClanUpdate,
    ) -> Result<Self, ApiError> {
        let updated_clan = diesel::update(clans::table.filter(clans::id.eq(clan_id)))
            .set(&clan)
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(updated_clan)
    }

    pub fn delete(
        conn: &mut DbConnection,
        clan_id: Uuid,
    ) -> Result<Clan, ApiError> {
        let clan = diesel::delete(clans::table.filter(clans::id.eq(clan_id)))
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(clan)
    }
}