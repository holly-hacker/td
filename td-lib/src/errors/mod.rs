//! Error types used by this crate

use thiserror::Error;

/// Errors that can occur when reading the task database.
#[derive(Error, Debug)]
pub enum DatabaseReadError {
    /// A database was loaded with an unsupported database version.
    #[error("unknown database version: {0}")]
    UnknownVersion(u8),

    /// A json deserialization error occured while reading the database structure.
    #[error("json deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// An IO error occured while reading the database file.
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}
