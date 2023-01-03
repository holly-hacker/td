//! The current version of the database as it is being developed.

use std::time::SystemTime;

use petgraph::stable_graph::StableDiGraph;
use serde::{Deserialize, Serialize};

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
    pub time_created: SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDependency;

impl super::DatabaseImpl for Database {
    const VERSION: u8 = 1;
}
