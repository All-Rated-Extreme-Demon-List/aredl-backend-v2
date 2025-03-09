use chrono::NaiveDateTime;
use diesel::{Connection, delete, ExpressionMethods, insert_into, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper, OptionalExtension};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use diesel::pg::Pg;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::clans::{Clan, ClanInvite, ClanMember};
use crate::schema::{clan_invites, clan_members, clans, users};
use crate::users::me::notifications::{Notification, NotificationType};
use crate::users::BaseDiscordUser;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Selectable, Queryable, ToSchema)]
#[diesel(table_name=clan_members, check_for_backend(Pg))]
pub struct ClanMemberAdd {
	/// Internal UUID of the clan to add the user to.
	pub clan_id: Uuid,
	/// Internal UUID of the user to add.
	pub user_id: Uuid
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Selectable, Queryable, ToSchema)]
#[diesel(table_name=clan_members, check_for_backend(Pg))]
pub struct ClanMemberDelete {
	/// Internal UUID of the clan to remove the user from.
	pub clan_id: Uuid,
	/// Internal UUID of the user to remove.
	pub user_id: Uuid
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Selectable, Queryable, ToSchema)]
#[diesel(table_name=clan_members, check_for_backend(Pg))]
pub struct ClanMemberUpdate {
	/// New Role of the user in the clan.
	pub role: i32
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Selectable, Queryable, ToSchema)]
#[diesel(table_name=clan_members, check_for_backend(Pg))]
pub struct ClanMemberInvite {
	/// Internal UUID of the user to invite
	pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Selectable, Insertable, Queryable, ToSchema)]
#[diesel(table_name=clan_invites, check_for_backend(Pg))]
pub struct ClanInviteCreate {
    /// Internal UUID of the clan to invite the user to.
    pub clan_id: Uuid,
    /// Internal UUID of the user to invite.
    pub user_id: Uuid,
    /// Internal UUID of the user who invited the user.
    pub invited_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Queryable, Selectable)]
#[diesel(table_name=clan_members)]
pub struct ClanMemberMeta {
	/// Role of the user in the clan.
	pub role: i32,
    /// Timestamp of when the user joined the clan.
    pub created_at: NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize, ToSchema, Queryable)]
pub struct ClanMemberResolved {
	#[serde(flatten)]
	pub user: BaseDiscordUser,
	#[serde(flatten)]
	pub member: ClanMemberMeta
}

impl ClanMember {

	pub fn find_all_clan_members(conn: &mut DbConnection, clan_id: Uuid) -> Result<Vec<ClanMemberResolved>, ApiError> {
        let members = clan_members::table
            .filter(clan_members::clan_id.eq(clan_id))
            .inner_join(users::table.on(clan_members::user_id.eq(users::id)))
            .select((
				BaseDiscordUser::as_select(),
				ClanMemberMeta::as_select()
			))
            .load::<ClanMemberResolved>(conn)?;
        Ok(members)
    }

	pub fn add_all(conn: &mut DbConnection, clan_id: Uuid, members: Vec<Uuid>) -> Result<Vec<Uuid>, ApiError> {
		let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {

            Self::add_members(clan_id, members.as_ref(), connection)?;

            let members = clan_members::table
				.filter(clan_members::clan_id.eq(clan_id))
				.select(clan_members::user_id)
				.load::<Uuid>(connection)?;

            Ok(members)
        })?;

		Ok(result)
	}

	pub fn remove_all(conn: &mut DbConnection, clan_id: Uuid, members: Vec<Uuid>) -> Result<Vec<Uuid>, ApiError> {
		let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
			delete(clan_members::table)
				.filter(clan_members::clan_id.eq(clan_id))
				.filter(clan_members::user_id.eq_any(&members))
				.execute(connection)?;

			let members = clan_members::table
				.filter(clan_members::clan_id.eq(clan_id))
				.select(clan_members::user_id)
				.load::<Uuid>(connection)?;

			Ok(members)
		})?;

		Ok(result)
	}

	pub fn set_all(conn: &mut DbConnection, clan_id: Uuid, members: Vec<Uuid>) -> Result<Vec<Uuid>, ApiError> {

        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            delete(clan_members::table)
                .filter(clan_members::clan_id.eq(clan_id))
                .execute(connection)?;

            Self::add_members(clan_id, members.as_ref(), connection)?;

            Ok(members)
        })?;

        Ok(result)
    }

	pub fn edit_member_role(conn: &mut DbConnection, clan_id: Uuid, user_id: Uuid, member: ClanMemberUpdate) -> Result<Self, ApiError> {
		let member = diesel::update(clan_members::table)
			.filter(clan_members::clan_id.eq(clan_id))
			.filter(clan_members::user_id.eq(user_id))
			.set({clan_members::role.eq(member.role); clan_members::updated_at.eq(chrono::Utc::now().naive_utc())})
			.returning(Self::as_select())
			.get_result(conn)?;
		Ok(member)
	}

	fn add_members(clan_id: Uuid, members: &Vec<Uuid>, connection: &mut DbConnection) -> Result<(), ApiError> {
        insert_into(clan_members::table)
            .values(
                members.into_iter().map(|member| (
                    clan_members::clan_id.eq(clan_id),
                    clan_members::user_id.eq(member)
                )).collect::<Vec<_>>()
            )
            .execute(connection)?;
        Ok(())
    }

	

}

impl ClanInvite {
	pub fn create(conn: &mut DbConnection, invite: ClanInviteCreate) -> Result<ClanInvite, ApiError> {
		let user_in_clan = clan_members::table
			.filter(clan_members::user_id.eq(invite.user_id))
			.select(clan_members::user_id)
			.first::<Uuid>(conn)
			.optional()?;
		if user_in_clan.is_some() {
			return Err(ApiError::new(400, "This user is already in a clan"));
		}

		let invited_by = users::table
			.filter(users::id.eq(invite.invited_by))
			.select(users::global_name)
			.first::<String>(conn)?;

		let clan = clans::table
				.filter(clans::id.eq(invite.clan_id))
				.first::<Clan>(conn)?;

		let result = conn.transaction::<_, ApiError, _>(|connection| {
            let invite = insert_into(clan_invites::table)
				.values(invite)
				.returning(ClanInvite::as_select())
				.get_result(connection)?;


            let content = format!("{} invited you to join {}", invited_by, clan.global_name);
            Notification::create(connection, invite.user_id, content, NotificationType::Info)?;

            Ok(invite)
        })?;
		
		Ok(result)
	}
}