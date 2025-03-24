use std::sync::Arc;
use actix_web::web;
use diesel::{ExpressionMethods, RunQueryDsl};
use diesel::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::schema::{aredl_levels, aredl_records, users};
use crate::users::BaseUser;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[derive(Serialize, Deserialize, Clone, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=aredl_levels)]
pub struct BaseLevel {
    /// Internal level UUID
    pub id: Uuid, 
    /// Name of the level in the game. If multiple levels share the same name, their creator's name is appended at the end. 2P levels both have (2P) or (Solo) appended at the end.
    pub name: String, 
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=aredl_levels)]
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
#[diesel(table_name=aredl_levels)]
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
    pub nlw_tier: Option<String>
}

#[derive(Serialize, Deserialize, Insertable, ToSchema, Debug)]
#[diesel(table_name=aredl_levels)]
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

#[derive(Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name=aredl_levels)]
pub struct LevelUpdate {
    /// The 1-indexed position of the level on the list.
    pub position: Option<i32>,
    /// Name of the level in the game. If multiple levels share the same name, their creator's name is appended at the end. 2P levels both have (2P) or (Solo) appended at the end.
    pub name: Option<String>,
    /// Internal user UUID of the person who published the level in the game.
    pub publisher_id: Option<Uuid>,
    /// Whether this level has been rerated to insane and is now in the legacy list, or not.
    pub legacy: Option<bool>,
    /// Whether this is the 2P version of a level or not.
    pub two_player: Option<bool>,
    /// Newground's song ID for the level.
    pub song: Option<i32>,
    /// Tags that describe the level. Includes gameplay, length, version, etc.. tags.
    pub tags: Option<Vec<Option<String>>>,
    /// Description of the level.
    pub description: Option<String>,
}

pub trait RecordSubmitter {}

impl RecordSubmitter for Uuid {}
impl RecordSubmitter for BaseUser {}

#[derive(Serialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=aredl_records)]
pub struct Record<T>
where T : RecordSubmitter
{
    /// Internal record UUID
    pub id: Uuid,
    /// User who submitted the record.
    pub submitted_by: T,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link to the completion.
    pub video_url: String,
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
    /// Record that is the verification for the level.
    pub verification: Option<Record<BaseUser>>,
}


impl Level {
    pub fn find_all(db: web::Data<Arc<DbAppState>>) -> Result<Vec<Self>, ApiError>{
        let levels = aredl_levels::table
            .select(Level::as_select())
            .order(aredl_levels::position)
            .load::<Self>(&mut db.connection()?)?;
        Ok(levels)
    }

    pub fn create(db: web::Data<Arc<DbAppState>>, level: LevelPlace) -> Result<Self, ApiError> {
        let level = diesel::insert_into(aredl_levels::table)
            .values(level)
            .returning(Self::as_select())
            .get_result(&mut db.connection()?)?;
        Ok(level)
    }

    pub fn update(db: web::Data<Arc<DbAppState>>, id: Uuid, level: LevelUpdate) -> Result<Self, ApiError> {
        let level = diesel::update(aredl_levels::table)
            .set(level)
            .filter(aredl_levels::id.eq(id))
            .returning(Self::as_select())
            .get_result(&mut db.connection()?)?;
        Ok(level)
    }
}

impl ResolvedLevel {
    pub fn find(db: web::Data<Arc<DbAppState>>, id: Uuid) -> Result<Self, ApiError> {
        let (level, publisher) = aredl_levels::table
            .filter(aredl_levels::id.eq(id))
            .inner_join(users::table.on(aredl_levels::publisher_id.eq(users::id)))
            .select(
                (Level::as_select(), BaseUser::as_select())
            )
            .first::<(Level, BaseUser)>(&mut db.connection()?)?;

        let verification = aredl_records::table
            .filter(aredl_records::level_id.eq(id))
            .filter(aredl_records::placement_order.eq(0))
            .inner_join(users::table.on(aredl_records::submitted_by.eq(users::id)))
            .select(
                (Record::<Uuid>::as_select(), BaseUser::as_select())
            )
            .first::<(Record<Uuid>, BaseUser)>(&mut db.connection()?)
            .optional()?;

        let verification = verification.map(
            |record| Record::from(record)
        );

        let resolved_level = Self::from_data(level, publisher, verification);
        Ok(resolved_level)
    }

    pub fn from_data(level: Level, publisher: BaseUser, verification: Option<Record<BaseUser>>) -> Self {
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
            verification,
        }
    }
}

impl From<(Record<Uuid>, BaseUser)> for Record<BaseUser> {
    fn from((record, user): (Record<Uuid>, BaseUser)) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
        }
    }
}