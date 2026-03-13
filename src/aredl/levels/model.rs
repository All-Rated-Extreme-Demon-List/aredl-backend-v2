use crate::app_data::db::DbConnection;
use crate::aredl::levels::records::LevelResolvedRecord;
use crate::aredl::records::Record;
use crate::error_handler::ApiError;
use crate::schema::aredl::{levels, position_history, position_history_full_view, records};
use crate::schema::users;
use crate::users::{BaseUser, BaseUserWithBanLevel};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::NullableExpressionMethods;
use diesel::{ExpressionMethods, RunQueryDsl};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=levels)]
pub struct BaseLevel {
    /// Internal level UUID
    pub id: Uuid,
    /// Name of the level in the game. If multiple levels share the same name, their creator's name is appended at the end. 2P levels both have (2P) or (Solo) appended at the end.
    pub name: String,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=levels)]
pub struct ExtendedBaseLevel {
    /// Internal level UUID
    pub id: Uuid,
    /// Name of the level in the game. If multiple levels share the same name, their creator's name is appended at the end. 2P levels both have (2P) or (Solo) appended at the end.
    pub name: String,
    /// Level ID in the game. May not be unique for 2P levels.
    pub level_id: i32,
    /// Whether this is the 2P version of a level or not.
    pub two_player: bool,
    /// The 1-indexed position of the level on the list.
    pub position: i32,
    /// Points awarded for completing the level.
    pub points: i32,
    /// Whether this level has been rerated to insane and is now in the legacy list, or not.
    pub legacy: bool,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=levels)]
pub struct Level {
    /// Internal level UUID
    pub id: Uuid,
    /// Name of the level in the game. If multiple levels share the same name, their creator's name is appended at the end. 2P levels both have (2P) or (Solo) appended at the end.
    pub name: String,
    /// The 1-indexed position of the level on the list.
    pub position: i32,
    /// Internal user UUID of the person who published the level in the game.
    pub publisher_id: Uuid,
    /// Points awarded for completing the level.
    pub points: i32,
    /// Whether this level has been rerated to insane and is now in the legacy list, or not.
    pub legacy: bool,
    /// Level ID in the game. May not be unique for 2P levels.
    pub level_id: i32,
    /// Whether this is the 2P version of a level or not.
    pub two_player: bool,
    /// Tags that describe the level. Includes gameplay, length, version, etc.. tags.
    pub tags: Vec<Option<String>>,
    /// Description of the level.
    pub description: Option<String>,
    /// Newground's song ID for the level.
    pub song: Option<i32>,
    /// Enjoyment rating for the level, fetched from EDEL (Extreme Demon Enjoyments List).
    pub edel_enjoyment: Option<f64>,
    /// Whether the EDEL enjoyment rating is pending (considered unreliable) or not.
    pub is_edel_pending: bool,
    /// GDDL tier for the level, fetched from GDDL (GD Demon Ladder).
    pub gddl_tier: Option<f64>,
    /// NLW tier for the level, fetched from NLW (Non List Worthy).
    pub nlw_tier: Option<String>,
}

#[derive(Serialize, Deserialize, Insertable, ToSchema, Debug)]
#[diesel(table_name=levels)]
pub struct LevelPlace {
    /// The 1-indexed position of the level on the list.
    pub position: i32,
    /// Name of the level in the game. If multiple levels share the same name, their creator's name is appended at the end. 2P levels both have (2P) or (Solo) appended at the end.
    pub name: String,
    /// Internal user UUID of the person who published the level in the game.
    pub publisher_id: Uuid,
    /// Whether this level has been rerated to insane and is now in the legacy list, or not.
    pub legacy: bool,
    /// Level ID in the game. May not be unique for 2P levels.
    pub level_id: i32,
    /// Whether this is the 2P version of a level or not.
    pub two_player: bool,
    /// Newground's song ID for the level.
    pub song: Option<i32>,
    /// Tags that describe the level. Includes gameplay, length, version, etc.. tags.
    pub tags: Option<Vec<Option<String>>>,
    /// Description of the level.
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, AsChangeset, ToSchema, Debug)]
#[diesel(table_name=levels)]
pub struct LevelUpdate {
    /// The 1-indexed position of the level on the list.
    pub position: Option<i32>,
    /// Name of the level in the game. If multiple levels share the same name, their creator's name is appended at the end. 2P levels both have (2P) or (Solo) appended at the end.
    pub name: Option<String>,
    /// Internal user UUID of the person who published the level in the game.
    pub publisher_id: Option<Uuid>,
    /// Whether this level has been rerated to insane and is now in the legacy list, or not.
    pub legacy: Option<bool>,
    /// Level ID in the game. May not be unique for 2P levels.
    pub level_id: Option<i32>,
    /// Whether this is the 2P version of a level or not.
    pub two_player: Option<bool>,
    /// Newground's song ID for the level.
    pub song: Option<i32>,
    /// Tags that describe the level. Includes gameplay, length, version, etc.. tags.
    pub tags: Option<Vec<Option<String>>>,
    /// Description of the level.
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelQueryOptions {
    pub exclude_legacy: Option<bool>,
    pub at: Option<DateTime<Utc>>,
}

// Level struct that has publisher and verification resolved
#[derive(Serialize, Debug, ToSchema)]
pub struct ResolvedLevel {
    /// Internal level UUID
    pub id: Uuid,
    /// The 1-indexed position of the level on the list.
    pub position: i32,
    /// Name of the level in the game. If multiple levels share the same name, their creator's name is appended at the end. 2P levels both have (2P) or (Solo) appended at the end.
    pub name: String,
    /// Points awarded for completing the level.
    pub points: i32,
    /// Whether this level has been rerated to insane and is now in the legacy list, or not.
    pub legacy: bool,
    /// Level ID in the game. May not be unique for 2P levels.
    pub level_id: i32,
    /// Whether this is the 2P version of a level or not.
    pub two_player: bool,
    /// Tags that describe the level. Includes gameplay, length, version, etc.. tags.
    pub tags: Vec<Option<String>>,
    /// Description of the level.
    pub description: Option<String>,
    /// Newground's song ID for the level.
    pub song: Option<i32>,
    /// Enjoyment rating for the level, fetched from EDEL (Extreme Demon Enjoyments List).
    pub edel_enjoyment: Option<f64>,
    /// Whether the EDEL enjoyment rating is pending (considered unreliable) or not.
    pub is_edel_pending: bool,
    /// GDDL tier for the level, fetched from GDDL (GD Demon Ladder).
    pub gddl_tier: Option<f64>,
    /// NLW tier for the level, fetched from NLW (Non List Worthy).
    pub nlw_tier: Option<String>,
    /// User who published the level.
    pub publisher: BaseUser,
    /// Records that are marked as verifications for the level.
    pub verifications: Vec<LevelResolvedRecord>,
}

impl Level {
    pub fn find_all(
        conn: &mut DbConnection,
        query: LevelQueryOptions,
    ) -> Result<Vec<Self>, ApiError> {
        let mut levels = if let Some(at) = query.at {
            Self::get_all_levels_at_timestamp(conn, at)?
        } else {
            levels::table
                .select(Level::as_select())
                .order(levels::position)
                .load::<Self>(conn)?
        };

        if let Some(true) = query.exclude_legacy {
            levels.retain(|level| !level.legacy);
        }

        Ok(levels)
    }

    fn get_all_levels_at_timestamp(
        conn: &mut DbConnection,
        at: DateTime<Utc>,
    ) -> Result<Vec<Self>, ApiError> {
        let cutoff = position_history::table
            .filter(position_history::created_at.le(at))
            .count()
            .get_result::<i64>(conn)? as i32;

        if cutoff == 0 {
            return Ok(Vec::new());
        }

        let mut time_machine_levels = position_history_full_view::table
            .filter(position_history_full_view::ord.le(cutoff))
            .filter(position_history_full_view::position.is_not_null())
            .distinct_on(position_history_full_view::affected_level)
            .order_by((
                position_history_full_view::affected_level.asc(),
                position_history_full_view::ord.desc(),
            ))
            .inner_join(levels::table.on(position_history_full_view::affected_level.eq(levels::id)))
            .select((
                levels::id,
                levels::name,
                position_history_full_view::position.assume_not_null(),
                levels::publisher_id,
                levels::points,
                position_history_full_view::legacy,
                levels::level_id,
                levels::two_player,
                levels::tags,
                levels::description,
                levels::song,
                levels::edel_enjoyment,
                levels::is_edel_pending,
                levels::gddl_tier,
                levels::nlw_tier,
            ))
            .load::<Self>(conn)?;

        time_machine_levels.sort_by_key(|level| level.position);
        Ok(time_machine_levels)
    }

    pub fn create(conn: &mut DbConnection, level: LevelPlace) -> Result<Self, ApiError> {
        let level = diesel::insert_into(levels::table)
            .values(level)
            .returning(Self::as_select())
            .get_result(conn)?;
        Ok(level)
    }

    pub fn update(conn: &mut DbConnection, id: Uuid, level: LevelUpdate) -> Result<Self, ApiError> {
        let level = diesel::update(levels::table)
            .set(level)
            .filter(levels::id.eq(id))
            .returning(Self::as_select())
            .get_result(conn)?;
        Ok(level)
    }
}

impl ResolvedLevel {
    pub fn find(conn: &mut DbConnection, id: Uuid) -> Result<Self, ApiError> {
        let (level, publisher) = levels::table
            .filter(levels::id.eq(id))
            .inner_join(users::table.on(levels::publisher_id.eq(users::id)))
            .select((Level::as_select(), BaseUserWithBanLevel::as_select()))
            .first::<(Level, BaseUserWithBanLevel)>(conn)?;

        let verifications_rows = records::table
            .filter(records::level_id.eq(id))
            .filter(records::is_verification.eq(true))
            .order(records::achieved_at.asc())
            .inner_join(users::table.on(records::submitted_by.eq(users::id)))
            .select((Record::as_select(), BaseUserWithBanLevel::as_select()))
            .load::<(Record, BaseUserWithBanLevel)>(conn)?;

        let verifications = verifications_rows
            .into_iter()
            .map(|(record, user)| {
                LevelResolvedRecord::from_data(
                    record,
                    BaseUser::from_base_user_with_ban_level(user),
                )
            })
            .collect::<Vec<_>>();

        let resolved_level = Self::from_data(
            level,
            BaseUser::from_base_user_with_ban_level(publisher),
            verifications,
        );
        Ok(resolved_level)
    }

    pub fn from_data(
        level: Level,
        publisher: BaseUser,
        verifications: Vec<LevelResolvedRecord>,
    ) -> Self {
        Self {
            id: level.id,
            position: level.position,
            name: level.name,
            points: level.points,
            legacy: level.legacy,
            level_id: level.level_id,
            two_player: level.two_player,
            tags: level.tags,
            description: level.description,
            song: level.song,
            edel_enjoyment: level.edel_enjoyment,
            is_edel_pending: level.is_edel_pending,
            gddl_tier: level.gddl_tier,
            nlw_tier: level.nlw_tier,
            publisher,
            verifications,
        }
    }
}
