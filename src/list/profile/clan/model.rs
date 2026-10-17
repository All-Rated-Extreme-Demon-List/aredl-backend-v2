use crate::app_data::db::DbConnection;
use crate::clans::Clan;
use crate::error_handler::ApiError;
use crate::list::levels::records::LevelResolvedRecordExtended;
use crate::list::levels::ExtendedBaseLevel;
use crate::list::levels::LevelStatus;
use crate::list::profile::ProfileLevelCreators;
use crate::list::profile::ProfileLevelPublisher;
use crate::list::profile::ProfileRecordWithCount;
use crate::list::records::Record;
use crate::list::List;
use crate::schema::{
    clan_member_points, clan_members, clans, clans_created_levels, clans_leaderboard, levels,
    min_placement_clans_records, records, users,
};
use crate::users::{AccountStatus, BaseUser, ExtendedBaseUser};
use diesel::pg::Pg;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=clans_leaderboard)]
pub struct Rank {
    pub rank: i32,
    pub extremes_rank: i32,
    pub hardest_rank: i32,
    pub level_points: i32,
    pub extremes: i32,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=clan_member_points, check_for_backend(Pg))]
pub struct ClanMemberPointsEntry {
    pub clan_id: Uuid,
    pub submitted_by: Uuid,
    pub completed_levels: i64,
    pub contributed_points: f64,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ResolvedClanMemberPoints {
    /// The member of the clan for this entry.
    pub member: ExtendedBaseUser,
    /// How many levels this member has completed.
    pub completed_levels: i64,
    /// The total points contributed by this member to the clan. For each record, the member's contribution is the level's given points divided by how many clan members completed it.
    pub contributed_points: f64,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ClanProfileResolved {
    /// This profile's clan.
    pub clan: Clan,
    /// Rank of the clan in the clans leaderboard.
    pub rank: Option<Rank>,
    /// Records of users from this clan. (Only the clan's first victor/verifier)
    pub records: Vec<ProfileRecordWithCount>,
    /// Points contributed by each member of the clan. (For each record, the member's contribution is the level's given points divided by how many clan members completed it)
    pub members_points: Vec<ResolvedClanMemberPoints>,
    /// Levels created by users from this clan.
    pub created: Vec<ProfileLevelCreators>,
    /// Levels published by users from this clan.
    pub published: Vec<ProfileLevelPublisher>,
}

impl ResolvedClanMemberPoints {
    pub fn from_data(entry: &ClanMemberPointsEntry, member: ExtendedBaseUser) -> Self {
        Self {
            member,
            completed_levels: entry.completed_levels,
            contributed_points: entry.contributed_points,
        }
    }
}

impl ClanProfileResolved {
    pub fn find(conn: &mut DbConnection, list: List, clan_id: Uuid) -> Result<Self, ApiError> {
        let clan = clans::table
            .filter(clans::id.eq(clan_id))
            .select(Clan::as_select())
            .first(conn)?;

        let rank = clans_leaderboard::table
            .filter(clans_leaderboard::list_id.eq(list))
            .filter(clans_leaderboard::clan_id.eq(clan_id))
            .select(Rank::as_select())
            .first(conn)
            .optional()?;

        let records = min_placement_clans_records::table
            .filter(min_placement_clans_records::list_id.eq(list))
            .filter(min_placement_clans_records::clan_id.eq(clan_id))
            .inner_join(
                records::table
                    .inner_join(users::table)
                    .inner_join(levels::table)
                    .on(records::id
                        .eq(min_placement_clans_records::id)
                        .and(records::list_id.eq(min_placement_clans_records::list_id))),
            )
            .select((
                Record::as_select(),
                min_placement_clans_records::completion_count,
                ExtendedBaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ))
            .order_by(levels::position.asc())
            .load::<(Record, i64, ExtendedBaseUser, ExtendedBaseLevel)>(conn)?
            .into_iter()
            .map(ProfileRecordWithCount::from)
            .collect();

        let members_points = clan_member_points::table
            .filter(clan_member_points::list_id.eq(list))
            .filter(clan_member_points::clan_id.eq(clan_id))
            .inner_join(users::table)
            .order_by((
                clan_member_points::contributed_points.desc(),
                clan_member_points::completed_levels.desc(),
                users::global_name.asc(),
                users::id.asc(),
            ))
            .select((
                ClanMemberPointsEntry::as_select(),
                ExtendedBaseUser::as_select(),
            ))
            .load::<(ClanMemberPointsEntry, ExtendedBaseUser)>(conn)?
            .into_iter()
            .map(|(entry, member)| ResolvedClanMemberPoints::from_data(&entry, member))
            .collect();

        let created_rows: Vec<(ExtendedBaseLevel, BaseUser)> = clans_created_levels::table
            .filter(clans_created_levels::list_id.eq(list))
            .filter(clans_created_levels::clan_id.eq(clan_id))
            .inner_join(levels::table)
            .inner_join(users::table)
            .filter(users::account_status.lt(AccountStatus::Redacted))
            .order_by((
                clans_created_levels::order_pos.asc(),
                users::global_name.asc(),
                users::id.asc(),
            ))
            .select((ExtendedBaseLevel::as_select(), BaseUser::as_select()))
            .load(conn)?;

        let created = ProfileLevelCreators::group_by_level(created_rows);

        let published: Vec<ProfileLevelPublisher> = levels::table
            .filter(levels::list_id.eq(list))
            .inner_join(users::table.inner_join(clan_members::table))
            .filter(clan_members::clan_id.eq(clan_id))
            .filter(users::account_status.lt(AccountStatus::Redacted))
            .order_by(levels::position.asc())
            .select((ExtendedBaseLevel::as_select(), BaseUser::as_select()))
            .load::<(ExtendedBaseLevel, BaseUser)>(conn)?
            .into_iter()
            .map(ProfileLevelPublisher::from)
            .collect();

        Ok(Self {
            clan,
            rank,
            records,
            members_points,
            created,
            published,
        })
    }

    pub fn find_records_for_level(
        conn: &mut DbConnection,
        list: List,
        clan_id: Uuid,
        level_id: Uuid,
    ) -> Result<Vec<LevelResolvedRecordExtended>, ApiError> {
        Ok(records::table
            .filter(records::list_id.eq(list))
            .filter(records::level_id.eq(level_id))
            .inner_join(users::table.inner_join(clan_members::table))
            .inner_join(levels::table)
            .filter(clan_members::clan_id.eq(clan_id))
            .filter(users::account_status.le(AccountStatus::SemiUnranked))
            .filter(levels::status.ne(LevelStatus::Removed))
            .order(records::achieved_at.asc())
            .select((Record::as_select(), ExtendedBaseUser::as_select()))
            .load::<(Record, ExtendedBaseUser)>(conn)?
            .into_iter()
            .map(LevelResolvedRecordExtended::from)
            .collect())
    }
}
