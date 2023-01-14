use std::ops::{Index, IndexMut};

use petgraph::stable_graph::NodeIndex;
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
