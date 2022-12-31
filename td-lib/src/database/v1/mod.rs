//! The current version of the database as it is being developed.

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Database {
    pub items: Vec<String>,
}

impl super::DatabaseImpl for Database {
    const VERSION: u8 = 1;
}
