//! The current version of the database as it is being developed.

use petgraph::stable_graph::StableDiGraph;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::utils::generate_unique_id;

/// The in-memory representation of the database
#[derive(Debug, Clone, Default)]
pub struct Database {
    /// The graph of tasks in this database.
    ///
    /// This uses a StableDiGraph to keep a stable order, which means insertions and removals will
    /// not cause large changes to the database file.
    pub tasks: StableDiGraph<Task, TaskDependency>,
}

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

/// The database model as stored to disk.
#[derive(Deserialize, Serialize)]
struct DatabaseDiskModel {
    tasks: Vec<TaskDiskModel>,
}

impl From<Database> for DatabaseDiskModel {
    fn from(value: Database) -> Self {
        let mut list = vec![];

        // collect nodes
        for node_idx in value.tasks.node_indices() {
            let node_weight = value.tasks[node_idx].clone();
            list.push((node_idx, TaskDiskModel::new(node_weight)));
        }

        // collect edges
        for edge_idx in value.tasks.edge_indices() {
            let (start_index, end_index) = value
                .tasks
                .edge_endpoints(edge_idx)
                .expect("each edge should be connected");

            let end_id = list
                .iter()
                .find_map(|x| (x.0 == end_index).then(|| x.1.task.id.clone()))
                .expect("should be able to find end node");
            let start_node = list
                .iter_mut()
                .find(|x| x.0 == start_index)
                .expect("should be able to find start node");

            start_node.1.dependencies.push(end_id);
        }

        Self {
            tasks: list.into_iter().map(|x| x.1).collect(),
        }
    }
}

impl From<DatabaseDiskModel> for Database {
    fn from(value: DatabaseDiskModel) -> Self {
        let mut graph = StableDiGraph::new();
        let mut id_index_map = vec![];

        // store nodes
        for task in &value.tasks {
            let id = task.task.id.clone();
            let index = graph.add_node(task.task.clone());
            id_index_map.push((id, index));
        }

        // store edges
        for task in &value.tasks {
            let source_id = task.task.id.clone();
            for target_id in task.dependencies.iter().cloned() {
                let source_index = id_index_map.iter().find(|x| x.0 == source_id).unwrap().1;
                let target_index = id_index_map.iter().find(|x| x.0 == target_id).unwrap().1;

                graph.add_edge(source_index, target_index, TaskDependency::new());
            }
        }

        Self { tasks: graph }
    }
}

#[derive(Deserialize, Serialize)]
struct TaskDiskModel {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dependencies: Vec<String>,

    #[serde(flatten)]
    task: Task,
}

impl TaskDiskModel {
    pub fn new(task: Task) -> Self {
        Self {
            task,
            dependencies: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// A unique id for this task
    id: String,
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
            id: generate_unique_id(),
            title,
            time_created,
            time_started: None,
            time_completed: None,
            tags: vec![],
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskDependency {}

impl TaskDependency {
    pub fn new() -> Self {
        Self::default()
    }
}

impl super::DatabaseImpl for Database {
    const VERSION: u8 = 1;
}
