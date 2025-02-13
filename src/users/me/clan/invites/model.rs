use diesel::{Connection, delete, ExpressionMethods, insert_into, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::clans::{Clan, ClanInvite};
use crate::clans::members::ClanMemberAdd;
use crate::schema::{clan_members, clan_invites, clans};

#[derive(Debug, Serialize, Deserialize, ToSchema, Queryable)]
pub struct ClanInviteResolved {
	/// Invite received by the user.
	#[serde(flatten)]
	pub invite: ClanInvite,
	/// Clan the user is invited to.
	pub clan: Clan,
}

impl ClanInvite {

	pub fn find_all_me_invites(conn: &mut DbConnection, user_id: Uuid) -> Result<Vec<ClanInviteResolved>, ApiError> {
        let invites = clan_invites::table
            .filter(clan_invites::user_id.eq(user_id))
            .inner_join(clans::table.on(clan_invites::clan_id.eq(clans::id)))
            .select((ClanInvite::as_select(), Clan::as_select()))
            .load::<ClanInviteResolved>(conn)?;
        Ok(invites)
    }

	pub fn accept_invite(conn: &mut DbConnection, invite_id: Uuid) -> Result<(), ApiError> {
		conn.transaction(|connection| -> Result<(), ApiError> {
			let invite = clan_invites::table
				.filter(clan_invites::id.eq(invite_id))
				.select(ClanInvite::as_select())
				.first::<ClanInvite>(connection)?;

			delete(clan_invites::table)
				.filter(clan_invites::user_id.eq(invite.user_id))
				.execute(connection)?;

			insert_into(clan_members::table)
				.values(ClanMemberAdd {
					clan_id: invite.clan_id,
					user_id: invite.user_id
				})
				.execute(connection)?;

			Ok(())
		})?;

		Ok(())
	}

	pub fn reject_invite(conn: &mut DbConnection, invite_id: Uuid) -> Result<(), ApiError> {
		delete(clan_invites::table)
			.filter(clan_invites::id.eq(invite_id))
			.execute(conn)?;
		Ok(())
	}

}