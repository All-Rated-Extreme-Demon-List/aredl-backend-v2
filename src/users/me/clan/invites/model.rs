use crate::app_data::db::DbConnection;
use crate::auth::Authenticated;
use crate::clans::members::ClanMemberAdd;
use crate::clans::{Clan, ClanInvite};
use crate::error_handler::ApiError;
use crate::schema::{clan_invites, clan_members, clans, users};
use crate::users::me::notifications::{Notification, NotificationType};
use crate::users::BaseUser;
use diesel::{
    delete, insert_into, Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, Queryable)]
pub struct ClanInviteResolved {
    /// Invite received by the user.
    #[serde(flatten)]
    pub invite: ClanInvite,
    /// Clan the user is invited to.
    pub clan: Clan,
}

impl ClanInvite {
    pub fn find_all_me_invites(
        conn: &mut DbConnection,
        user_id: Uuid,
    ) -> Result<Vec<ClanInviteResolved>, ApiError> {
        let invites = clan_invites::table
            .filter(clan_invites::user_id.eq(user_id))
            .inner_join(clans::table.on(clan_invites::clan_id.eq(clans::id)))
            .select((ClanInvite::as_select(), Clan::as_select()))
            .load::<ClanInviteResolved>(conn)?;
        Ok(invites)
    }

    pub fn accept_invite(
        conn: &mut DbConnection,
        invite_id: Uuid,
        authenticated: Authenticated,
    ) -> Result<(), ApiError> {
        conn.transaction(|connection| -> Result<(), ApiError> {
            let invite = clan_invites::table
                .filter(clan_invites::id.eq(invite_id))
                .select(ClanInvite::as_select())
                .first::<ClanInvite>(connection)?;

            if invite.user_id != authenticated.user_id {
                return Err(ApiError::new(
                    403,
                    "You can not accept an invite that's not yours",
                ));
            }

            let clan = clans::table
                .filter(clans::id.eq(invite.clan_id))
                .select(Clan::as_select())
                .first::<Clan>(connection)?;

            let user = users::table
                .filter(users::id.eq(invite.user_id))
                .select(BaseUser::as_select())
                .first::<BaseUser>(connection)?;

            delete(clan_invites::table)
                .filter(clan_invites::user_id.eq(invite.user_id))
                .execute(connection)?;

            insert_into(clan_members::table)
                .values(ClanMemberAdd {
                    clan_id: invite.clan_id,
                    user_id: invite.user_id,
                })
                .execute(connection)?;

            let content = format!(
                "{} accepted your invite to join {}",
                user.global_name, clan.global_name
            );
            Notification::create(
                connection,
                invite.invited_by,
                content,
                NotificationType::Success,
            )?;
            Ok(())
        })?;

        Ok(())
    }

    pub fn reject_invite(
        conn: &mut DbConnection,
        invite_id: Uuid,
        authenticated: Authenticated,
    ) -> Result<(), ApiError> {
        let invite = clan_invites::table
            .filter(clan_invites::id.eq(invite_id))
            .select(ClanInvite::as_select())
            .first::<ClanInvite>(conn)?;

        if invite.user_id != authenticated.user_id {
            return Err(ApiError::new(
                403,
                "You can not reject an invite that's not yours",
            ));
        }
        delete(clan_invites::table)
            .filter(clan_invites::id.eq(invite_id))
            .execute(conn)?;
        Ok(())
    }
}
