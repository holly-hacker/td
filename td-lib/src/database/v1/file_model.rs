use petgraph::stable_graph::StableDiGraph;
use serde::{Deserialize, Serialize};

use super::*;

/// The database model as stored to disk.
#[derive(Deserialize, Serialize)]
pub struct DatabaseDiskModel {
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
        let mut id_index_map = HashMap::new();

        // store nodes
        for task in &value.tasks {
            let id = task.task.id.clone();
            let index = graph.add_node(task.task.clone());
            id_index_map.insert(id, index);
        }

        // store edges
        for task in &value.tasks {
            let source_id = task.task.id.clone();
            for target_id in task.dependencies.iter() {
                let source_index = id_index_map[&source_id];
                let target_index = id_index_map[target_id];

                graph.add_edge(source_index, target_index, TaskDependency);
            }
        }

        Self {
            tasks: graph,
            task_id_to_index: id_index_map,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct TaskDiskModel {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dependencies: Vec<TaskId>,

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
