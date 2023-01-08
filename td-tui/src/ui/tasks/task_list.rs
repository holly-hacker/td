use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use td_lib::{
    database::{DatabaseInfo, Task, TaskDependency},
    petgraph::graph::NodeIndex,
    time::OffsetDateTime,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

use crate::{
    keybinds::{
        KEYBIND_TASK_ADD_DEPENDENCY, KEYBIND_TASK_ADD_TAG, KEYBIND_TASK_DELETE,
        KEYBIND_TASK_MARK_DONE, KEYBIND_TASK_NEW,
    },
    ui::{
        constants::{
            BOLD, COMPLETED_TASK, FG_DIM, FG_GREEN, FG_RED, FG_WHITE, ITALIC, LIST_HIGHLIGHT_STYLE,
            LIST_STYLE,
        },
        modal::{list_search::ListSearchModal, text_input::TextInputModal},
        task_info::TaskInfoDisplay,
        AppState, Component, FrameLocalStorage,
    },
};

pub struct BasicTaskList {
    index: usize,
    task_popup: TextInputModal,
    tag_popup: TextInputModal,
    search_box_depend_on: ListSearchModal<NodeIndex>,
    newest_first: bool,
}

impl BasicTaskList {
    pub fn new(reverse: bool) -> Self {
        Self {
            index: 0,
            task_popup: TextInputModal::new("Enter new task".to_string()),
            tag_popup: TextInputModal::new("Enter new tag".to_string()),
            search_box_depend_on: ListSearchModal::new(
                "Choose which task to depend on".to_string(),
            ),
            newest_first: reverse,
        }
    }

    fn get_sorted_task_list(&self, state: &AppState) -> Vec<(NodeIndex, Task)> {
        let mut tasks = state
            .database
            .tasks
            .node_indices()
            .map(|i| {
                (
                    i,
                    state
                        .database
                        .tasks
                        .node_weight(i)
                        .cloned()
                        .expect("should find weight for NodeIndex"),
                )
            })
            .collect::<Vec<_>>();

        tasks.sort_by(|a, b| a.1.time_created.cmp(&b.1.time_created));
        if self.newest_first {
            tasks.reverse();
        }

        tasks
    }

    fn task_to_span(&self, state: &AppState, task_index: &NodeIndex, task: &Task) -> Spans {
        let mut spans = vec![];

        let dependents_count = state
            .database
            .tasks
            .edges_directed(*task_index, td_lib::petgraph::Direction::Incoming)
            .count();
        if dependents_count > 0 {
            spans.push(Span::styled(
                format!("{:>2}⤣", dependents_count.to_string()),
                FG_GREEN.patch(BOLD),
            ));
        }

        let dependency_count = state
            .database
            .tasks
            .edges_directed(*task_index, td_lib::petgraph::Direction::Outgoing)
            .count();
        if dependency_count > 0 {
            spans.push(Span::styled(
                format!("{:>2}⤥", dependency_count.to_string()),
                FG_RED.patch(BOLD),
            ));
        }

        if dependency_count > 0 || dependents_count > 0 {
            spans.push(Span::raw(" "));
        }

        // add title
        let text_style = if task.time_completed.is_some() {
            LIST_STYLE.patch(COMPLETED_TASK)
        } else {
            LIST_STYLE
        };
        spans.push(Span::styled(task.title.clone(), text_style));

        // add tags
        for task in &task.tags {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(task.clone(), FG_DIM.patch(ITALIC)));
        }

        spans.into()
    }
}

impl Component for BasicTaskList {
    fn pre_render(&self, global_state: &AppState, frame_storage: &mut FrameLocalStorage) {
        // store currently selected task in frame storage
        let task_list = self.get_sorted_task_list(global_state);
        let selected_task_id = task_list.get(self.index).map(|x| x.0);
        frame_storage.selected_task_index = selected_task_id;

        self.task_popup.pre_render(global_state, frame_storage);
        self.tag_popup.pre_render(global_state, frame_storage);
        self.search_box_depend_on
            .pre_render(global_state, frame_storage);

        frame_storage.add_keybind("⇅", "Navigate list", task_list.len() >= 2);
        frame_storage.add_keybind(
            KEYBIND_TASK_MARK_DONE.to_string(),
            "Mask as done",
            selected_task_id.is_some(),
        );
        frame_storage.add_keybind(KEYBIND_TASK_NEW.to_string(), "New task", true);
        frame_storage.add_keybind(
            KEYBIND_TASK_DELETE.to_string(),
            "Delete task",
            selected_task_id.is_some(),
        );
        frame_storage.add_keybind(
            KEYBIND_TASK_ADD_TAG.to_string(),
            "Add tag",
            selected_task_id.is_some(),
        );
        frame_storage.add_keybind(
            KEYBIND_TASK_ADD_DEPENDENCY.to_string(),
            "Add dependency",
            true,
        );
    }

    fn render(
        &self,
        frame: &mut Frame<CrosstermBackend<Stdout>>,
        area: Rect,
        state: &AppState,
        frame_storage: &crate::ui::FrameLocalStorage,
    ) {
        let layout = Layout::default()
            .constraints([Constraint::Percentage(67), Constraint::Percentage(33)])
            .direction(Direction::Horizontal)
            .split(area);

        let list_area = layout[0];
        let info_area = layout[1];

        let task_list = self.get_sorted_task_list(state);

        // render the list
        let block = Block::default()
            .title(if self.newest_first {
                "Basic Task List"
            } else {
                "Basic Task List (oldest first)"
            })
            .borders(Borders::ALL)
            .border_style(FG_WHITE)
            .border_type(BorderType::Rounded);

        let list_items = task_list
            .iter()
            .map(|t| ListItem::new(self.task_to_span(state, &t.0, &t.1)))
            .collect::<Vec<_>>();
        let list = List::new(list_items)
            .block(block)
            .highlight_style(LIST_HIGHLIGHT_STYLE)
            .style(LIST_STYLE);
        let mut list_state = ListState::default();
        list_state.select((!task_list.is_empty()).then_some(self.index));
        frame.render_stateful_widget(list, list_area, &mut list_state);

        // render info
        TaskInfoDisplay.render(frame, info_area, state, frame_storage);

        // if needed, render popups
        self.task_popup.render(frame, area, state, frame_storage);
        self.tag_popup.render(frame, area, state, frame_storage);
        self.search_box_depend_on
            .render(frame, area, state, frame_storage);
    }

    fn process_input(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
        frame_storage: &FrameLocalStorage,
    ) -> bool {
        // check modals
        if self.task_popup.process_input(key, state, frame_storage)
            || self.tag_popup.process_input(key, state, frame_storage)
            || self
                .search_box_depend_on
                .process_input(key, state, frame_storage)
        {
            return true;
        }

        let tasks = self.get_sorted_task_list(state);

        if !tasks.is_empty() {
            self.index = self.index.clamp(0, tasks.len() - 1);
        }

        if self.task_popup.is_open() {
            // popup is open
            if key.code == KeyCode::Enter {
                if let Some(text) = self.task_popup.close() {
                    state.database.tasks.add_node(Task::create_now(text));

                    // TODO: error handling. show popup on failure to save?
                    let db_info: DatabaseInfo = (&state.database).into();
                    db_info.write(&state.path).unwrap();
                }
                true
            } else {
                false
            }
        } else if self.tag_popup.is_open() {
            // popup is open
            if key.code == KeyCode::Enter {
                if let Some(text) = self.tag_popup.close() {
                    let selected_task_id = tasks[self.index].0;
                    let selected_task = &mut state.database.tasks[selected_task_id];
                    selected_task.tags.push(text);

                    // TODO: error handling. show popup on failure to save?
                    let db_info: DatabaseInfo = (&state.database).into();
                    db_info.write(&state.path).unwrap();
                }
                true
            } else {
                false
            }
        } else if self.search_box_depend_on.is_open() {
            // popup is open
            if key.code == KeyCode::Enter {
                if let Some(selected_node) = self.search_box_depend_on.close() {
                    let current_node = tasks[self.index].0;

                    state.database.tasks.add_edge(
                        current_node,
                        selected_node,
                        TaskDependency::new(),
                    );

                    // TODO: error handling. show popup on failure to save?
                    let db_info: DatabaseInfo = (&state.database).into();
                    db_info.write(&state.path).unwrap();
                }

                true
            } else {
                false
            }
        } else {
            // take our own input
            match (key.code, key.modifiers) {
                (KeyCode::Char(KEYBIND_TASK_MARK_DONE), KeyModifiers::NONE) => {
                    let task = &mut state.database.tasks[tasks[self.index].0];
                    if task.time_completed.is_none() {
                        task.time_completed = Some(
                            OffsetDateTime::now_local()
                                .unwrap_or_else(|_| OffsetDateTime::now_utc()),
                        );
                    } else {
                        task.time_completed = None;
                    }

                    // TODO: error handling. show popup on failure to save?
                    let db_info: DatabaseInfo = (&state.database).into();
                    db_info.write(&state.path).unwrap();

                    true
                }
                (KeyCode::Char(KEYBIND_TASK_NEW), KeyModifiers::NONE) => {
                    self.task_popup.open();
                    true
                }
                (KeyCode::Char(KEYBIND_TASK_DELETE), KeyModifiers::NONE) => {
                    if !tasks.is_empty() {
                        // delete
                        state.database.tasks.remove_node(tasks[self.index].0);

                        // TODO: error handling. show popup on failure to save?
                        let db_info: DatabaseInfo = (&state.database).into();
                        db_info.write(&state.path).unwrap();
                    }

                    true
                }
                (KeyCode::Char(KEYBIND_TASK_ADD_TAG), KeyModifiers::NONE) => {
                    if !tasks.is_empty() {
                        // add tag to currently selected task
                        self.tag_popup.open();
                    }

                    true
                }
                (KeyCode::Char(KEYBIND_TASK_ADD_DEPENDENCY), KeyModifiers::NONE) => {
                    // link to other task
                    let selected = &tasks[self.index];
                    let tasks = tasks
                        .iter()
                        .filter(|t| t.0 != selected.0)
                        .filter(|candidate| {
                            !state.database.tasks.contains_edge(selected.0, candidate.0)
                        })
                        .map(|w| (w.0, w.1.title.clone()))
                        .collect();
                    self.search_box_depend_on.open(tasks);
                    true
                }
                (KeyCode::Up, KeyModifiers::NONE) => {
                    self.index = self.index.saturating_sub(1);
                    true
                }
                (KeyCode::Down, KeyModifiers::NONE) => {
                    if !tasks.is_empty() && self.index != tasks.len() - 1 {
                        self.index += 1;
                    }
                    true
                }
                _ => false,
            }
        }
    }
}
