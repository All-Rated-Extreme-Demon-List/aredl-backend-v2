use std::sync::Arc;
use actix_web::web;
use diesel::{Connection, delete, ExpressionMethods, insert_into, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use diesel::pg::Pg;
use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::users::BaseUser;
use crate::clans::ClanMember;
use crate::schema::{clans, clan_members, users, clan_invites};

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Selectable, Queryable, ToSchema)]
#[diesel(table_name=clan_members, check_for_backend(Pg))]
pub struct ClanMemberAdd {
	/// Internal UUID of the clan.
	pub clan_id: Uuid,
	/// Internal UUID of the user.
	pub user_id: Uuid
}

impl BaseUser {
    pub fn find_all_clan_members(conn: &mut DbConnection, clan_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let members = clan_members::table
            .filter(clan_members::clan_id.eq(clan_id))
            .inner_join(users::table.on(clan_members::user_id.eq(users::id)))
            .select(BaseUser::as_select())
            .load::<BaseUser>(conn)?;
        Ok(members)
    }

	pub fn remove_clan_member(conn: &mut DbConnection, clan_id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
		diesel::delete(clan_members::table)
			.filter(clan_members::clan_id.eq(clan_id))
			.filter(clan_members::user_id.eq(user_id))
			.execute(conn)?;
		Ok(())
	}

	pub fn add_clan_member(conn: &mut DbConnection, member: ClanMemberAdd) -> Result<(), ApiError> {
		diesel::insert_into(clan_members::table)
			.values(member)
			.execute(conn)?;
		Ok(())
	}
}