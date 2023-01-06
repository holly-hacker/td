pub mod v1;

use std::path::Path;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
// NOTE: this import should import the current version of the database schema
pub use v1::*;

use crate::errors::DatabaseReadError;

#[derive(Serialize, Deserialize)]
pub struct DatabaseInfo {
    version: u8,
    data: serde_json::Value,
}

impl DatabaseInfo {
    pub fn read(path: &Path) -> Result<Self, DatabaseReadError> {
        let file = std::fs::read(path)?;

        let db_info: DatabaseInfo = serde_json::from_slice(&file)?;

        Ok(db_info)
    }

    pub fn write(&self, path: &Path) -> Result<(), DatabaseReadError> {
        let json = serde_json::to_vec_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl Default for DatabaseInfo {
    fn default() -> Self {
        let db = Database::default();
        Self {
            version: Database::VERSION,
            data: serde_json::to_value(db).expect("new database should always be valid json"),
        }
    }
}

impl TryInto<Database> for DatabaseInfo {
    type Error = DatabaseReadError;

    // NOTE: migrations would happen here
    fn try_into(self) -> Result<Database, Self::Error> {
        if self.version != 1 {
            return Err(DatabaseReadError::UnknownVersion(self.version));
        }
        Ok(serde_json::from_value(self.data)?)
    }
}

impl From<&Database> for DatabaseInfo {
    fn from(value: &Database) -> Self {
        Self {
            version: Database::VERSION,
            data: serde_json::to_value(value).expect("Failed to serialize"),
        }
    }
}

trait DatabaseImpl: Default + Serialize + DeserializeOwned {
    const VERSION: u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn new_db_is_valid_json() {
        let db = v1::Database::default();
        serde_json::to_value(db).expect("new database should always be valid json");
    }
}
