use crate::app_data::db::DbConnection;
use crate::aredl::levels::records::LevelResolvedRecordExtended;
use crate::aredl::levels::ExtendedBaseLevel;
use crate::aredl::levels::LevelStatus;
use crate::aredl::records::{Record, ResolvedRecord};
use crate::error_handler::ApiError;
use crate::schema::{
    aredl::{
        country_created_levels, country_leaderboard, levels, min_placement_country_records, records,
    },
    users,
};
use crate::users::{BaseUser, ExtendedBaseUser};
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::{
    ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
};
use indexmap::map::Entry;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=country_leaderboard)]
pub struct Rank {
    pub rank: i32,
    pub extremes_rank: i32,
    pub level_points: i32,
    pub extremes: i32,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=min_placement_country_records, check_for_backend(Pg))]
pub struct CountryProfileRecord {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// Internal UUID of the submission this record is linked to.
    pub submission_id: Uuid,
    /// Internal UUID of the level the record is for.
    pub level_id: Uuid,
    /// Internal UUID of the user who submitted the record.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Whether the record is a verification or not.
    pub is_verification: bool,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Timestamp of when this record was achieved, used for ordering.
    pub achieved_at: DateTime<Utc>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
    /// How many member completed the same level.
    pub completion_count: i64,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ResolvedCountryProfileRecord {
    #[serde(flatten)]
    pub record: ResolvedRecord,
    pub completion_count: i64,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ResolvedCountryProfileLevel {
    #[serde(flatten)]
    pub level: ExtendedBaseLevel,
    /// The user who published the level.
    pub publisher: BaseUser,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ResolvedCountryProfileCreatedLevel {
    #[serde(flatten)]
    pub level: ExtendedBaseLevel,
    /// Users from this country who are listed as creators for the level.
    pub creators: Vec<BaseUser>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=country_created_levels, check_for_backend(Pg))]
pub struct CountryCreatedLevelEntry {
    pub country: i32,
    pub level_id: Uuid,
    pub creator_id: Uuid,
    pub order_pos: Option<i32>,
}

impl ResolvedRecord {
    pub fn from_country_data(
        record: CountryProfileRecord,
        level: ExtendedBaseLevel,
        user: ExtendedBaseUser,
    ) -> Self {
        Self {
            id: record.id,
            submission_id: record.submission_id,
            submitted_by: user,
            level,
            mobile: record.mobile,
            video_url: record.video_url,
            is_verification: record.is_verification,
            hide_video: record.hide_video,
            achieved_at: record.achieved_at,
            updated_at: record.updated_at,
            created_at: record.created_at,
        }
    }
}

impl ResolvedCountryProfileRecord {
    pub fn from_country_data(
        record: CountryProfileRecord,
        level: ExtendedBaseLevel,
        user: ExtendedBaseUser,
    ) -> Self {
        let completion_count = record.completion_count;
        Self {
            record: ResolvedRecord::from_country_data(record, level, user),
            completion_count,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CountryProfileResolved {
    /// Country of the profile. Uses the ISO 3166-1 numeric country code.
    pub country: i32,
    /// Rank of the country in the countries leaderboard.
    pub rank: Option<Rank>,
    /// Records of users from this country. (Only the country's first victor/verifier)
    pub records: Vec<ResolvedCountryProfileRecord>,
    /// Levels created by users from the country.
    pub created: Vec<ResolvedCountryProfileCreatedLevel>,
    /// Levels published by users from the country.
    pub published: Vec<ResolvedCountryProfileLevel>,
}

impl CountryProfileResolved {
    pub fn find(conn: &mut DbConnection, country: i32) -> Result<Self, ApiError> {
        let rank = country_leaderboard::table
            .filter(country_leaderboard::country.eq(country))
            .select(Rank::as_select())
            .first(conn)
            .optional()?;

        let records = min_placement_country_records::table
            .filter(min_placement_country_records::country.eq(country))
            .inner_join(users::table.on(users::id.eq(min_placement_country_records::submitted_by)))
            .inner_join(levels::table.on(levels::id.eq(min_placement_country_records::level_id)))
            .select((
                CountryProfileRecord::as_select(),
                ExtendedBaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ))
            .order_by(levels::position.asc())
            .load::<(CountryProfileRecord, ExtendedBaseUser, ExtendedBaseLevel)>(conn)?
            .into_iter()
            .map(|(record, user, level)| {
                ResolvedCountryProfileRecord::from_country_data(record, level, user)
            })
            .collect();

        let created_rows: Vec<(CountryCreatedLevelEntry, ExtendedBaseLevel, BaseUser)> =
            country_created_levels::table
                .filter(country_created_levels::country.eq(country))
                .inner_join(levels::table.on(levels::id.eq(country_created_levels::level_id)))
                .inner_join(users::table.on(users::id.eq(country_created_levels::creator_id)))
                .order_by((
                    country_created_levels::order_pos.asc(),
                    users::global_name.asc(),
                    users::id.asc(),
                ))
                .select((
                    CountryCreatedLevelEntry::as_select(),
                    ExtendedBaseLevel::as_select(),
                    BaseUser::as_select(),
                ))
                .load(conn)?;

        let mut created_by_level: IndexMap<Uuid, ResolvedCountryProfileCreatedLevel> =
            IndexMap::new();
        for (_, level, user) in created_rows {
            match created_by_level.entry(level.id) {
                Entry::Occupied(entry) => {
                    entry.into_mut().creators.push(user);
                }
                Entry::Vacant(entry) => {
                    entry.insert(ResolvedCountryProfileCreatedLevel {
                        level,
                        creators: vec![user],
                    });
                }
            }
        }
        let created = created_by_level.into_values().collect();

        let published = levels::table
            .inner_join(users::table.on(users::id.eq(levels::publisher_id)))
            .filter(users::country.eq(country))
            .order_by(levels::position.asc())
            .select((ExtendedBaseLevel::as_select(), BaseUser::as_select()))
            .load(conn)?
            .into_iter()
            .map(|(level, user)| ResolvedCountryProfileLevel {
                level,
                publisher: user,
            })
            .collect();

        Ok(Self {
            country,
            rank,
            records,
            created,
            published,
        })
    }

    pub fn find_records_for_level(
        conn: &mut DbConnection,
        country: i32,
        level_id: Uuid,
    ) -> Result<Vec<LevelResolvedRecordExtended>, ApiError> {
        records::table
            .filter(records::level_id.eq(level_id))
            .inner_join(users::table.on(records::submitted_by.eq(users::id)))
            .inner_join(levels::table.on(records::level_id.eq(levels::id)))
            .filter(users::country.eq(country))
            .filter(users::ban_level.eq(0))
            .filter(levels::status.ne(LevelStatus::Removed))
            .order(records::achieved_at.asc())
            .select((Record::as_select(), ExtendedBaseUser::as_select()))
            .load::<(Record, ExtendedBaseUser)>(conn)
            .map(|rows| {
                rows.into_iter()
                    .map(|(record, user)| LevelResolvedRecordExtended::from_data(record, user))
                    .collect()
            })
            .map_err(ApiError::from)
    }
}
