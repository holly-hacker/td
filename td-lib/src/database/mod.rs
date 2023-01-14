mod database_api;
pub mod database_file;
mod v1;

use serde::{de::DeserializeOwned, Serialize};
// NOTE: this import should import the current version of the database schema
pub use v1::*;

/// The current version of the database model.
pub const CURRENT_DATABASE_VERSION: u8 = Database::VERSION;

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
