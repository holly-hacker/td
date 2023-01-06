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
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

use super::task_info::TaskInfo;
use crate::ui::{
    constants::{LIST_HIGHLIGHT_STYLE, LIST_STYLE, STANDARD_STYLE_FG_WHITE},
    modal::{list_search::ListSearchModal, text_input::TextInputModal},
    AppState, Component,
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

    fn get_task_list<'state>(&self, state: &'state AppState) -> Vec<(NodeIndex, &'state Task)> {
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
}

impl Component for BasicTaskList {
    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, state: &AppState) {
        let layout = Layout::default()
            .constraints([Constraint::Percentage(67), Constraint::Percentage(33)])
            .direction(Direction::Horizontal)
            .split(area);

        let list_area = layout[0];
        let info_area = layout[1];

        let task_list = self.get_task_list(state);

        // render the list
        let block = Block::default()
            .title(if self.newest_first {
                "Basic Task List"
            } else {
                "Basic Task List (oldest first)"
            })
            .borders(Borders::ALL)
            .border_style(STANDARD_STYLE_FG_WHITE)
            .border_type(BorderType::Rounded);

        let list_items = task_list
            .iter()
            .map(|t| ListItem::new(t.1.title.clone()))
            .collect::<Vec<_>>();
        let list = List::new(list_items)
            .block(block)
            .highlight_style(LIST_HIGHLIGHT_STYLE)
            .style(LIST_STYLE);
        let mut list_state = ListState::default();
        list_state.select((!task_list.is_empty()).then_some(self.index));
        frame.render_stateful_widget(list, list_area, &mut list_state);

        // render info
        TaskInfo::new(task_list.get(self.index).map(|x| x.0)).render(frame, info_area, state);

        // if needed, render popups
        self.task_popup.render(frame, area, state);
        self.tag_popup.render(frame, area, state);
        self.search_box_depend_on.render(frame, area, state);
    }

    fn update(&mut self, key: KeyEvent, state: &mut AppState) -> bool {
        // check modals
        if self.task_popup.update(key, state)
            || self.tag_popup.update(key, state)
            || self.search_box_depend_on.update(key, state)
        {
            return true;
        }

        let tasks = self.get_task_list(state);

        if !tasks.is_empty() {
            self.index = self.index.clamp(0, tasks.len() - 1);
        }

        if self.task_popup.is_open() {
            // popup is open
            if key.code == KeyCode::Enter {
                if let Some(text) = self.task_popup.close() {
                    let time_created =
                        OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
                    let task = Task {
                        title: text,
                        time_created,
                        tags: vec![],
                    };
                    state.database.tasks.add_node(task);

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
                (KeyCode::Char('c'), KeyModifiers::NONE) => {
                    self.task_popup.open();
                    true
                }
                (KeyCode::Char('d'), KeyModifiers::NONE) => {
                    if !tasks.is_empty() {
                        // delete
                        state.database.tasks.remove_node(tasks[self.index].0);

                        // TODO: error handling. show popup on failure to save?
                        let db_info: DatabaseInfo = (&state.database).into();
                        db_info.write(&state.path).unwrap();
                    }

                    true
                }
                (KeyCode::Char('t'), KeyModifiers::NONE) => {
                    if !tasks.is_empty() {
                        // add tag to currently selected task
                        self.tag_popup.open();
                    }

                    true
                }
                (KeyCode::Char('l'), KeyModifiers::NONE) => {
                    // link to other task
                    let selected = tasks[self.index];
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
                    if self.index != tasks.len() - 1 {
                        self.index += 1;
                    }
                    true
                }
                _ => false,
            }
        }
    }
}
