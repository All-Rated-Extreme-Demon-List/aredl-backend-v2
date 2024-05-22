use strum_macros::{EnumString, Display};

#[derive(Clone, EnumString, Display)]
#[strum(serialize_all="snake_case")]
pub enum Permission {
    LevelModify
}