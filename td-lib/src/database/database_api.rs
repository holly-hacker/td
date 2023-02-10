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

        &self.graph[node_index]
    }
}

impl IndexMut<&TaskId> for Database {
    fn index_mut(&mut self, task_id: &TaskId) -> &mut Self::Output {
        let node_index = self.get_node_index(task_id);
        let Some(node_index) = node_index else {
            panic!("Index not found");
        };

        &mut self.graph[node_index]
    }
}

impl Database {
    /// Adds a new task to the database.
    pub fn add_task(&mut self, task: Task) {
        let id = task.id.clone();
        let index = self.graph.add_node(task);
        self.task_id_to_index.insert(id, index);
    }

    /// Removes a task from the database. If the given task id was not found, no changes are made.
    pub fn remove_task(&mut self, task_id: &TaskId) {
        self.task_id_to_index.remove(task_id);
        let Some(task_index) = self.get_node_index(task_id) else {return;};
        self.graph.remove_node(task_index);
    }

    /// Get all tasks in the database.
    pub fn get_all_tasks(&self) -> impl Iterator<Item = &Task> + '_ {
        self.graph.node_weights()
    }

    /// Add a task dependency between 2 tasks. This indicates that one task depends on another.
    pub fn add_dependency(&mut self, from: &TaskId, to: &TaskId) {
        let from_index = self
            .get_node_index(from)
            .expect("should be able to resolve task id");
        let to_index = self
            .get_node_index(to)
            .expect("should be able to resolve task id");

        self.graph.add_edge(from_index, to_index, TaskDependency);
    }

    /// Gets all the tasks the given task depends on.
    pub fn get_dependencies(&self, source: &TaskId) -> impl Iterator<Item = &Task> + '_ {
        let source_index = self
            .get_node_index(source)
            .expect("should be able to resolve task id");

        self.graph
            .edges_directed(source_index, Direction::Outgoing)
            .map(|edge| edge.target())
            .map(|target| &self.graph[target])
    }

    /// Gets all the tasks that depend on the given task.
    pub fn get_inverse_dependencies(&self, target: &TaskId) -> impl Iterator<Item = &Task> + '_ {
        let target_index = self
            .get_node_index(target)
            .expect("should be able to resolve task id");

        self.graph
            .edges_directed(target_index, Direction::Incoming)
            .map(|edge| edge.source())
            .map(|source| &self.graph[source])
    }

    fn get_node_index(&self, task_id: &TaskId) -> Option<NodeIndex> {
        self.task_id_to_index.get(task_id).copied().or_else(|| {
            // this fallback check exists in case we add a new node and it isn't in the cache.
            // this check should be removed when insertion of new tasks is managed here.
            for node_index in self.graph.node_indices() {
                let weight = &self.graph[node_index];
                if &weight.id == task_id {
                    return Some(node_index);
                }
            }
            None
        })
    }
}

impl Task {
    /// Create a new, empty task with the given title.
    #[must_use]
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

    /// Gets the internal ID of this task.
    #[must_use]
    pub fn id(&self) -> &TaskId {
        &self.id
    }
}
