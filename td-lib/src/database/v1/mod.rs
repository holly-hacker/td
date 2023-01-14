//! The current version of the database, starting at v0.1.

mod file_model;

use std::collections::HashMap;

use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use self::file_model::*;

/// The in-memory representation of the database
#[derive(Debug, Clone, Default)]
pub struct Database {
    /// The graph of tasks in this database.
    ///
    /// This uses a StableDiGraph to keep a stable order, which means insertions and removals will
    /// not cause large changes to the database file.
    pub tasks: StableDiGraph<Task, TaskDependency>,

    pub(crate) task_id_to_index: HashMap<TaskId, NodeIndex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// A unique id for this task
    pub id: TaskId,
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

#[derive(Debug, Clone, Default)]
pub struct TaskDependency;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(String);

// -- end public structs --

impl Serialize for Database {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let model: DatabaseDiskModel = self.clone().into();
        model.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Database {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let model = DatabaseDiskModel::deserialize::<D>(deserializer)?;
        Ok(model.into())
    }
}

impl TaskId {
    // TODO: take iterator of existing ids to ensure no collisions are generated
    pub(crate) fn new() -> Self {
        const SAFE_ALPHABET: [char; 56] = [
            '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j',
            'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
            'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U',
            'V', 'W', 'X', 'Y', 'Z',
        ];

        Self(nanoid::nanoid!(8, &SAFE_ALPHABET))
    }
}

impl super::DatabaseImpl for Database {
    const VERSION: u8 = 1;
}
