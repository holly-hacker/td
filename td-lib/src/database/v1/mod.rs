//! The current version of the database as it is being developed.

use petgraph::stable_graph::StableDiGraph;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// TODO: maybe want to have separate disk model and in-memory model? currently relying on petgraph's internal structure
// I can implement that using a custom serialize/deserialize implementation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Database {
    next_task_id: usize,

    /// The list of tasks in this database.
    ///
    /// This uses an IndexMap to keep a stable order, which means insertions and removals in the
    /// internal database will not cause large changes to the database file.
    pub tasks: StableDiGraph<Task, TaskDependency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub title: String,
    pub time_created: OffsetDateTime,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
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
