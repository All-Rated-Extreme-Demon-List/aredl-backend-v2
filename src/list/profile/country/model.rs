use crate::app_data::db::DbConnection;
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
    country_created_levels, country_leaderboard, levels,
    min_placement_country_records as country_records, records, users,
};
use crate::users::{AccountStatus, BaseUser, ExtendedBaseUser};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=country_leaderboard)]
pub struct Rank {
    pub rank: i32,
    pub extremes_rank: i32,
    pub hardest_rank: i32,
    pub level_points: i32,
    pub extremes: i32,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CountryProfileResolved {
    /// Country of the profile. Uses the ISO 3166-1 numeric country code.
    pub country: i32,
    /// Rank of the country in the countries leaderboard.
    pub rank: Option<Rank>,
    /// Records of users from this country. (Only the country's first victor/verifier)
    pub records: Vec<ProfileRecordWithCount>,
    /// Levels created by users from the country.
    pub created: Vec<ProfileLevelCreators>,
    /// Levels published by users from the country.
    pub published: Vec<ProfileLevelPublisher>,
}

impl CountryProfileResolved {
    pub fn find(conn: &mut DbConnection, list: List, country: i32) -> Result<Self, ApiError> {
        let rank = country_leaderboard::table
            .filter(country_leaderboard::list_id.eq(list))
            .filter(country_leaderboard::country.eq(country))
            .select(Rank::as_select())
            .first(conn)
            .optional()?;

        let records = country_records::table
            .filter(country_records::list_id.eq(list))
            .filter(country_records::country.eq(country))
            .inner_join(
                records::table
                    .inner_join(users::table)
                    .inner_join(levels::table)
                    .on(records::id
                        .eq(country_records::id)
                        .and(records::list_id.eq(country_records::list_id))),
            )
            .order_by(levels::position.asc())
            .select((
                Record::as_select(),
                country_records::completion_count,
                ExtendedBaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ))
            .load::<(Record, i64, ExtendedBaseUser, ExtendedBaseLevel)>(conn)?
            .into_iter()
            .map(ProfileRecordWithCount::from)
            .collect();

        let created_rows: Vec<(ExtendedBaseLevel, BaseUser)> = country_created_levels::table
            .filter(country_created_levels::list_id.eq(list))
            .filter(country_created_levels::country.eq(country))
            .inner_join(levels::table)
            .inner_join(users::table)
            .filter(users::account_status.lt(AccountStatus::Redacted))
            .order_by((
                country_created_levels::order_pos.asc(),
                users::global_name.asc(),
                users::id.asc(),
            ))
            .select((ExtendedBaseLevel::as_select(), BaseUser::as_select()))
            .load(conn)?;

        let created = ProfileLevelCreators::group_by_level(created_rows);

        let published = levels::table
            .filter(levels::list_id.eq(list))
            .inner_join(users::table)
            .filter(users::country.eq(country))
            .filter(users::account_status.lt(AccountStatus::Redacted))
            .order_by(levels::position.asc())
            .select((ExtendedBaseLevel::as_select(), BaseUser::as_select()))
            .load::<(ExtendedBaseLevel, BaseUser)>(conn)?
            .into_iter()
            .map(ProfileLevelPublisher::from)
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
        list: List,
        country: i32,
        level_id: Uuid,
    ) -> Result<Vec<LevelResolvedRecordExtended>, ApiError> {
        let records = records::table
            .filter(records::list_id.eq(list))
            .filter(records::level_id.eq(level_id))
            .inner_join(users::table)
            .inner_join(levels::table)
            .filter(users::country.eq(country))
            .filter(users::account_status.le(AccountStatus::SemiUnranked))
            .filter(levels::status.ne(LevelStatus::Removed))
            .order(records::achieved_at.asc())
            .select((Record::as_select(), ExtendedBaseUser::as_select()))
            .load::<(Record, ExtendedBaseUser)>(conn)?
            .into_iter()
            .map(LevelResolvedRecordExtended::from)
            .collect();

        Ok(records)
    }
}
