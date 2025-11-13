use crate::arepl::levels::ExtendedBaseLevel;
use crate::arepl::packs::{BasePack, PackWithTierResolved};
use crate::arepl::packtiers::BasePackTier;
use crate::clans::Clan;
use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{
    arepl::{
        completed_packs, levels, levels_created, pack_tiers, packs, records, user_leaderboard,
    },
    clan_members, clans, roles, user_roles,
};
use crate::users::{Role, User};
use chrono::{DateTime, Utc};
use diesel::{
    ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=user_leaderboard)]
pub struct Rank {
    /// Rank of the user in the global leaderboard, sorted by total points (including packs).
    pub rank: i32,
    /// Rank of the user in the global leaderboard, sorted by total points (excluding packs).
    pub raw_rank: i32,
    /// Rank of the user in the global leaderboard, sorted by count of extremes completed.
    pub extremes_rank: i32,
    /// Rank of the user in the country leaderboard, sorted by total points (including packs).
    pub country_rank: i32,
    /// Rank of the user in the country leaderboard, sorted by total points (excluding packs).
    pub country_raw_rank: i32,
    /// Rank of the user in the country leaderboard, sorted by count of extremes completed.
    pub country_extremes_rank: i32,
    /// Total points of the user, including pack points.
    pub total_points: i32,
    /// Pack points of the user.
    pub pack_points: i32,
    /// Count of extremes the user has completed.
    pub extremes: i32,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema, QueryableByName)]
#[diesel(table_name=records)]
pub struct ProfileRecord {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    #[serde(skip_serializing)]
    pub is_verification: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ProfileRecordResolved {
    #[serde(flatten)]
    pub record: ProfileRecord,
    pub level: ExtendedBaseLevel,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ProfileResolved {
    #[serde(flatten)]
    pub user: User,
    /// The clan the user is in.
    pub clan: Option<Clan>,
    /// The roles the user has.
    pub roles: Vec<Role>,
    /// Leaderboard ranks of the user.
    pub rank: Option<Rank>,
    /// Packs the user has completed.
    pub packs: Vec<PackWithTierResolved>,
    /// Records the user has submitted.
    pub records: Vec<ProfileRecordResolved>,
    /// Verifications of the user.
    pub verified: Vec<ProfileRecordResolved>,
    /// Levels the user is listed as a creator of.
    pub created: Vec<ExtendedBaseLevel>,
    /// Levels the user has published in game.
    pub published: Vec<ExtendedBaseLevel>,
}

impl ProfileResolved {
    pub fn from_str(conn: &mut DbConnection, user_id: &str) -> Result<Self, ApiError> {
        let user = User::from_str(conn, user_id)?;
        Self::from_user(conn, user)
    }

    pub fn from_user(conn: &mut DbConnection, user: User) -> Result<Self, ApiError> {
        if User::is_banned(user.id.clone(), conn)? {
            return Err(ApiError::new(
                403,
                "This user has been banned from the list.".into(),
            ));
        }
        let clan = clans::table
            .inner_join(clan_members::table.on(clans::id.eq(clan_members::clan_id)))
            .filter(clan_members::user_id.eq(user.id))
            .select(Clan::as_select())
            .first::<Clan>(conn)
            .optional()?;

        let roles = roles::table
            .inner_join(user_roles::table.on(user_roles::role_id.eq(roles::id)))
            .filter(user_roles::user_id.eq(user.id))
            .order(roles::privilege_level.desc())
            .select(Role::as_select())
            .load::<Role>(conn)?;

        let rank = user_leaderboard::table
            .filter(user_leaderboard::user_id.eq(user.id))
            .select(Rank::as_select())
            .first(conn)
            .optional()?;

        let full_records = records::table
            .filter(records::submitted_by.eq(user.id))
            .inner_join(levels::table.on(levels::id.eq(records::level_id)))
            .order(levels::position.asc())
            .select((ProfileRecord::as_select(), ExtendedBaseLevel::as_select()))
            .load::<(ProfileRecord, ExtendedBaseLevel)>(conn)?
            .into_iter()
            .map(|(record, level)| ProfileRecordResolved { record, level })
            .collect::<Vec<_>>();

        let (verified, records) = full_records
            .into_iter()
            .partition(|record| record.record.is_verification);

        let mut created = levels::table
            .inner_join(levels_created::table.on(levels_created::level_id.eq(levels::id)))
            .order(levels::position.asc())
            .filter(levels_created::user_id.eq(user.id))
            .select(ExtendedBaseLevel::as_select())
            .load::<ExtendedBaseLevel>(conn)?;

        let published = levels::table
            .filter(levels::publisher_id.eq(user.id))
            .order(levels::position.asc())
            .select(ExtendedBaseLevel::as_select())
            .load::<ExtendedBaseLevel>(conn)?;

        let published_without_creators_list: Vec<ExtendedBaseLevel> = levels::table
            .left_outer_join(levels_created::table.on(levels_created::level_id.eq(levels::id)))
            .filter(levels::publisher_id.eq(user.id))
            .filter(levels_created::level_id.is_null())
            .order(levels::position.asc())
            .select(ExtendedBaseLevel::as_select())
            .load(conn)?;

        created.extend(published_without_creators_list);
        created.sort_by_key(|lvl| lvl.position);

        let packs = packs::table
            .inner_join(completed_packs::table.on(completed_packs::pack_id.eq(packs::id)))
            .inner_join(pack_tiers::table.on(pack_tiers::id.eq(packs::tier)))
            .filter(completed_packs::user_id.eq(user.id))
            .order(pack_tiers::placement.asc())
            .select((BasePack::as_select(), BasePackTier::as_select()))
            .load::<(BasePack, BasePackTier)>(conn)?
            .into_iter()
            .map(|(pack, tier)| PackWithTierResolved { pack, tier })
            .collect::<Vec<_>>();

        Ok(Self {
            user,
            clan,
            roles,
            rank,
            packs,
            records,
            verified,
            created,
            published,
        })
    }
}
