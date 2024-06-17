use std::fmt::Formatter;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Deserializer};
use serde::de::{Error, Visitor};
use uuid::Uuid;
use crate::db;
use crate::schema::aredl_levels;

pub struct LevelId(Uuid);

impl<'de> Deserialize<'de> for LevelId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_str(LevelIdVisitor)
    }
}

struct LevelIdVisitor;

impl<'de> Visitor<'de> for LevelIdVisitor {
    type Value = LevelId;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a valid Uuid or GD id")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
        match Uuid::parse_str(v) {
            Ok(uuid) => Ok(LevelId(uuid)),
            Err(_) => LevelId::resolve_gd_id(v)
        }
    }
}

impl LevelId {

    fn resolve_gd_id<E>(s: &str) -> Result<Self, E> where E: Error {
        let (parsed_id, two_player) = if s.ends_with("_2p") {
            (s[..s.len() - 3].parse::<i32>(), true)
        } else {
            (s.parse::<i32>(), false)
        };
        let id = parsed_id.map_err(|_| E::custom(format!("Failed to parse {}", s)))?;
        let resolved_id = aredl_levels::table
            .filter(aredl_levels::level_id.eq(id))
            .filter(aredl_levels::two_player.eq(two_player))
            .select(aredl_levels::id)
            .first::<Uuid>(&mut db::connection().map_err(|_| E::custom(format!("Failed to resolve {}", s)))?)
            .map_err(|_| E::custom(format!("Failed to resolve {}", s)))?;
        Ok(LevelId(resolved_id))
    }
}

impl Into<Uuid> for LevelId {
    fn into(self) -> Uuid {
        self.0
    }
}