use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::users::{Role, User};
use crate::aredl::packs::{BasePack, PackWithTierResolved};
use crate::aredl::packtiers::BasePackTier;
use crate::aredl::levels::ExtendedBaseLevel;
use crate::schema::{aredl_levels, aredl_levels_created, aredl_pack_tiers, aredl_packs, aredl_records, roles, user_roles, users};
use crate::custom_schema::{aredl_completed_packs, aredl_user_leaderboard};

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=aredl_user_leaderboard)]
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
#[diesel(table_name=aredl_records)]
pub struct ProfileRecord {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    #[serde(skip_serializing)]
    pub placement_order: i32,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: NaiveDateTime,
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
    pub fn find(conn: &mut DbConnection, id: Uuid) -> Result<Self, ApiError> {
        let user = users::table
            .filter(users::id.eq(id))
            .select(User::as_select())
            .get_result::<User>(conn)?;

        let roles = roles::table
            .inner_join(user_roles::table.on(user_roles::role_id.eq(roles::id)))
            .filter(user_roles::user_id.eq(id))
            .order(roles::privilege_level.desc())
            .select(Role::as_select())
            .load::<Role>(conn)?;

        let rank = aredl_user_leaderboard::table
            .filter(aredl_user_leaderboard::user_id.eq(id))
            .select(Rank::as_select())
            .first(conn)
            .optional()?;

        let full_records = aredl_records::table
            .filter(aredl_records::submitted_by.eq(id))
            .inner_join(aredl_levels::table.on(aredl_levels::id.eq(aredl_records::level_id)))
            .order(aredl_levels::position.asc())
            .select((ProfileRecord::as_select(), ExtendedBaseLevel::as_select()))
            .load::<(ProfileRecord, ExtendedBaseLevel)>(conn)?
            .into_iter()
            .map(|(record, level)| ProfileRecordResolved {
                record, level
            })
            .collect::<Vec<_>>();

        let (records, verified) = full_records.into_iter()
            .partition(|record| record.record.placement_order != 0);

        let created = aredl_levels::table
            .inner_join(aredl_levels_created::table.on(aredl_levels_created::level_id.eq(aredl_levels::id)))
            .order(aredl_levels::position.asc())
            .filter(aredl_levels_created::user_id.eq(id))
            .select(ExtendedBaseLevel::as_select())
            .load::<ExtendedBaseLevel>(conn)?;

        let published = aredl_levels::table
            .filter(aredl_levels::publisher_id.eq(id))
            .order(aredl_levels::position.asc())
            .select(ExtendedBaseLevel::as_select())
            .load::<ExtendedBaseLevel>(conn)?;

        let packs = aredl_packs::table
            .inner_join(aredl_completed_packs::table.on(aredl_completed_packs::pack_id.eq(aredl_packs::id)))
            .inner_join(aredl_pack_tiers::table.on(aredl_pack_tiers::id.eq(aredl_packs::tier)))
            .filter(aredl_completed_packs::user_id.eq(id))
            .order(aredl_pack_tiers::placement.asc())
            .select((BasePack::as_select(), BasePackTier::as_select()))
            .load::<(BasePack, BasePackTier)>(conn)?
            .into_iter()
            .map(|(pack, tier)| PackWithTierResolved { pack, tier })
            .collect::<Vec<_>>();

        Ok(Self { user, roles, rank, packs, records, verified, created, published })
    }
}