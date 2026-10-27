use chrono::{DateTime, Utc};
use indexmap::map::Entry;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    list::{
        levels::ExtendedBaseLevel,
        records::{Record, ResolvedRecord},
    },
    users::{BaseUser, ExtendedBaseUser},
};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
/// A resolved record for a specific level (ommits the level field compared to ResolvedRecord).
pub struct ProfileRecordResolved {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// The level the record is for.
    pub level: ExtendedBaseLevel,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds. Only present for platformer records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_time: Option<i64>,
    /// Whether this completion time should be displayed and used for sorting on the level's completions
    pub completion_time_verified: bool,
    /// Whether the record is a verification or not.
    pub is_verification: bool,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Timestamp of when the record was achieved
    pub achieved_at: DateTime<Utc>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

impl From<(Record, ExtendedBaseLevel)> for ProfileRecordResolved {
    fn from((record, level): (Record, ExtendedBaseLevel)) -> Self {
        Self {
            id: record.id,
            level,
            mobile: record.mobile,
            video_url: record.video_url,
            completion_time: record.completion_time,
            completion_time_verified: record.completion_time_verified,
            is_verification: record.is_verification,
            hide_video: record.hide_video,
            achieved_at: record.achieved_at,
            updated_at: record.updated_at,
            created_at: record.created_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ProfileRecordWithCount {
    #[serde(flatten)]
    pub record: ResolvedRecord,
    /// How many users from this clan/country have completed this level
    pub completion_count: i64,
}

impl From<(Record, i64, ExtendedBaseUser, ExtendedBaseLevel)> for ProfileRecordWithCount {
    fn from(
        (record, completion_count, user, level): (Record, i64, ExtendedBaseUser, ExtendedBaseLevel),
    ) -> Self {
        Self {
            record: ResolvedRecord::from((record, user, level)),
            completion_count,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ProfileLevelCreators {
    #[serde(flatten)]
    pub level: ExtendedBaseLevel,
    /// Users from this clan/country who are listed as creators for the level.
    pub creators: Vec<BaseUser>,
}

impl ProfileLevelCreators {
    pub fn group_by_level(rows: Vec<(ExtendedBaseLevel, BaseUser)>) -> Vec<Self> {
        let mut created_by_level: IndexMap<Uuid, Self> = IndexMap::new();
        for (level, user) in rows {
            match created_by_level.entry(level.id) {
                Entry::Occupied(entry) => {
                    entry.into_mut().creators.push(user);
                }
                Entry::Vacant(entry) => {
                    entry.insert(Self {
                        level,
                        creators: vec![user],
                    });
                }
            }
        }
        created_by_level.into_values().collect()
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ProfileLevelPublisher {
    #[serde(flatten)]
    pub level: ExtendedBaseLevel,
    /// The user from this clan/country who published the level.
    pub publisher: BaseUser,
}

impl From<(ExtendedBaseLevel, BaseUser)> for ProfileLevelPublisher {
    fn from((level, user): (ExtendedBaseLevel, BaseUser)) -> Self {
        Self {
            level,
            publisher: user,
        }
    }
}
