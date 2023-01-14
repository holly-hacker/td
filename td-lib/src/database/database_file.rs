//! Contains a version-agnostic wrapper structure around the version-specific database structure.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{Database, DatabaseImpl};
use crate::errors::DatabaseReadError;

/// A version-agnostic container for a database structure.
#[derive(Serialize, Deserialize)]
pub struct DatabaseFile {
    /// The expected database version.
    pub version: u8,
    data: serde_json::Value,
}

impl DatabaseFile {
    /// Read the database file from disk in json format.
    pub fn read(path: &Path) -> Result<Self, DatabaseReadError> {
        let file = std::fs::read(path)?;

        let db_info: DatabaseFile = serde_json::from_slice(&file)?;

        Ok(db_info)
    }

    /// Write the database file to disk in json format.
    pub fn write(&self, path: &Path) -> Result<(), DatabaseReadError> {
        let json = serde_json::to_vec_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl Default for DatabaseFile {
    fn default() -> Self {
        let db = Database::default();
        Self {
            version: Database::VERSION,
            data: serde_json::to_value(db).expect("new database should always be valid json"),
        }
    }
}

impl TryInto<Database> for DatabaseFile {
    type Error = DatabaseReadError;

    // NOTE: migrations would happen here
    fn try_into(self) -> Result<Database, Self::Error> {
        if self.version != 1 {
            return Err(DatabaseReadError::UnknownVersion(self.version));
        }
        Ok(serde_json::from_value(self.data)?)
    }
}

impl From<&Database> for DatabaseFile {
    fn from(value: &Database) -> Self {
        Self {
            version: Database::VERSION,
            data: serde_json::to_value(value).expect("Failed to serialize"),
        }
    }
}
