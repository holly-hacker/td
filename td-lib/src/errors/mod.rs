use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseReadError {
    #[error("unknown database version: {0}")]
    UnknownVersion(u8),

    #[error("json deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}
