use std::env::VarError;
use std::fmt;
use std::fmt::{Formatter};
use serde::{Deserialize, Serialize};
use diesel::result::Error as DieselError;
use serde::de::StdError;

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationError {
    pub error_message: String,
}

impl MigrationError {
    pub fn new(error_message: &str) -> MigrationError {
        MigrationError {
            error_message: error_message.to_string(),
        }
    }
}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.error_message.as_str())
    }
}

impl From<DieselError> for MigrationError {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::DatabaseError(_, err) =>
                MigrationError::new(err.message()),
            DieselError::NotFound =>
                MigrationError::new("Record not found"),
            err =>
                MigrationError::new(&format!("Unknown Diesel error: {}", err)),
        }
    }
}

impl From<Box<dyn StdError + Send + Sync>> for MigrationError {
    fn from(value: Box<dyn StdError + Send + Sync>) -> Self {
        MigrationError::new(value.to_string().as_str())
    }
}

impl From<VarError> for MigrationError {
    fn from(value: VarError) -> Self {
        MigrationError::new(value.to_string().as_str())
    }
}