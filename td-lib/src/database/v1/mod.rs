//! The current version of the database as it is being developed.

use petgraph::stable_graph::StableDiGraph;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// TODO: maybe want to have separate disk model and in-memory model? currently relying on petgraph's internal structure
// I can implement that using a custom serialize/deserialize implementation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Database {
    /// The list of tasks in this database.
    ///
    /// This uses an IndexMap to keep a stable order, which means insertions and removals in the
    /// internal database will not cause large changes to the database file.
    pub tasks: StableDiGraph<Task, TaskDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// A short description of this task.
    pub title: String,
    /// When the task has been created.
    pub time_created: OffsetDateTime,
    /// If the task has been started, this is when that happened.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_started: Option<OffsetDateTime>,
    /// If the task has been completed, this is when that happened.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_completed: Option<OffsetDateTime>,
    /// A list of tags for this task.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl Task {
    pub fn create_now(title: String) -> Self {
        let time_created =
            OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        Self {
            title,
            time_created,
            time_started: None,
            time_completed: None,
            tags: vec![],
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TaskDependency {}

impl TaskDependency {
    pub fn new() -> Self {
        Self::default()
    }
}

impl super::DatabaseImpl for Database {
    const VERSION: u8 = 1;
}
