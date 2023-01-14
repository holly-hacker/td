use std::ops::{Index, IndexMut};

use petgraph::{stable_graph::NodeIndex, visit::EdgeRef, Direction};
use time::OffsetDateTime;

use super::*;

impl Index<&TaskId> for Database {
    type Output = Task;

    fn index(&self, task_id: &TaskId) -> &Self::Output {
        let node_index = self.get_node_index(task_id);
        let Some(node_index) = node_index else {
            panic!("Index not found");
        };

        &self.tasks[node_index]
    }
}

impl IndexMut<&TaskId> for Database {
    fn index_mut(&mut self, task_id: &TaskId) -> &mut Self::Output {
        let node_index = self.get_node_index(task_id);
        let Some(node_index) = node_index else {
            panic!("Index not found");
        };

        &mut self.tasks[node_index]
    }
}

impl Database {
    pub fn add_task(&mut self, task: Task) {
        let id = task.id.clone();
        let index = self.tasks.add_node(task);
        self.task_id_to_index.insert(id, index);
    }

    pub fn remove_task(&mut self, task_id: &TaskId) {
        self.task_id_to_index.remove(task_id);
        let Some(task_index) = self.get_node_index(task_id) else {return;};
        self.tasks.remove_node(task_index);
    }

    pub fn get_all_tasks(&self) -> impl Iterator<Item = &Task> + '_ {
        self.tasks.node_weights()
    }

    pub fn add_dependency(&mut self, from: &TaskId, to: &TaskId) {
        let from_index = self
            .get_node_index(from)
            .expect("should be able to resolve task id");
        let to_index = self
            .get_node_index(to)
            .expect("should be able to resolve task id");

        self.tasks.add_edge(from_index, to_index, TaskDependency);
    }

    /// Gets all the tasks the given task depends on
    pub fn get_dependencies(&self, source: &TaskId) -> impl Iterator<Item = &Task> + '_ {
        let source_index = self
            .get_node_index(source)
            .expect("should be able to resolve task id");

        self.tasks
            .edges_directed(source_index, Direction::Outgoing)
            .map(|edge| edge.target())
            .map(|target| &self.tasks[target])
    }

    /// Gets all the tasks depend on the given task
    pub fn get_inverse_dependencies(&self, target: &TaskId) -> impl Iterator<Item = &Task> + '_ {
        let target_index = self
            .get_node_index(target)
            .expect("should be able to resolve task id");

        self.tasks
            .edges_directed(target_index, Direction::Incoming)
            .map(|edge| edge.source())
            .map(|source| &self.tasks[source])
    }

    // TODO: make this private
    pub fn get_node_index(&self, task_id: &TaskId) -> Option<NodeIndex> {
        self.task_id_to_index.get(task_id).cloned().or_else(|| {
            // this fallback check exists in case we add a new node and it isn't in the cache.
            // this check should be removed when insertion of new tasks is managed here.
            for node_index in self.tasks.node_indices() {
                let weight = &self.tasks[node_index];
                if &weight.id == task_id {
                    return Some(node_index);
                }
            }
            None
        })
    }
}

impl Task {
    pub fn create_now(title: String) -> Self {
        let time_created =
            OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        Self {
            id: TaskId::new(),
            title,
            time_created,
            time_started: None,
            time_completed: None,
            tags: vec![],
        }
    }
}
