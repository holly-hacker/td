use std::{collections::HashSet, io::Stdout};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use td_lib::{
    database::{Task, TaskId},
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
        KEYBIND_TASK_MARK_DONE, KEYBIND_TASK_MARK_STARTED, KEYBIND_TASK_NEW, KEYBIND_TASK_RENAME,
    },
    ui::{
        constants::{
            BOLD, COMPLETED_TASK, FG_DIM, FG_GREEN, FG_RED, FG_WHITE, ITALIC, LIST_HIGHLIGHT_STYLE,
            LIST_STYLE, STARTED_TASK,
        },
        modal::{list_search::ListSearchModal, text_input::TextInputModal},
        task_info::TaskInfoDisplay,
        AppState, Component, FrameLocalStorage,
    },
};

pub struct BasicTaskList {
    index: usize,
    create_task_modal: TextInputModal,
    new_tag_modal: TextInputModal,
    rename_task_modal: TextInputModal,
    search_box_depend_on: ListSearchModal<TaskId>,
    newest_first: bool,
}

impl BasicTaskList {
    const SCROLL_PAGE_UP_DOWN: usize = 32;

    pub fn new(reverse: bool) -> Self {
        Self {
            index: 0,
            create_task_modal: TextInputModal::new("Create new task".to_string()),
            new_tag_modal: TextInputModal::new("Add new tag".to_string()),
            rename_task_modal: TextInputModal::new("Rename task".to_string()),
            search_box_depend_on: ListSearchModal::new(
                "Choose which task to depend on".to_string(),
            ),
            newest_first: reverse,
        }
    }

    fn get_sorted_task_list(&self, state: &AppState) -> Vec<Task> {
        let mut tasks = state
            .database
            .tasks
            .node_weights()
            .cloned()
            .collect::<Vec<_>>();

        tasks.sort_by(|a, b| a.time_created.cmp(&b.time_created));
        if self.newest_first {
            tasks.reverse();
        }

        tasks
    }

    fn task_to_span(&self, state: &AppState, task: &Task) -> Spans {
        let mut spans = vec![];

        let dependents_count = state.database.get_inverse_dependencies(&task.id).count();
        if dependents_count > 0 {
            spans.push(Span::styled(
                format!("{:>2}⤣", dependents_count.to_string()),
                FG_GREEN.patch(BOLD),
            ));
        }

        let unfullfilled_dependency_count = state
            .database
            .get_dependencies(&task.id)
            .filter(|t| t.time_completed.is_none())
            .count();

        if unfullfilled_dependency_count > 0 {
            spans.push(Span::styled(
                format!("{:>2}⤥", unfullfilled_dependency_count.to_string()),
                FG_RED.patch(BOLD),
            ));
        }

        if unfullfilled_dependency_count > 0 || dependents_count > 0 {
            spans.push(Span::raw(" "));
        }

        // add title
        let text_style = if task.time_completed.is_some() {
            LIST_STYLE.patch(COMPLETED_TASK)
        } else if task.time_started.is_some() {
            LIST_STYLE.patch(STARTED_TASK)
        } else {
            LIST_STYLE
        };
        spans.push(Span::styled(task.title.clone(), text_style));

        // add tags
        for tag in &task.tags {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(tag.clone(), FG_DIM.patch(ITALIC)));
        }

        spans.into()
    }
}

impl Component for BasicTaskList {
    fn pre_render(&self, global_state: &AppState, frame_storage: &mut FrameLocalStorage) {
        // store currently selected task in frame storage
        let task_list = self.get_sorted_task_list(global_state);
        frame_storage.selected_task_id = task_list.get(self.index).map(|x| x.id.clone());

        self.create_task_modal
            .pre_render(global_state, frame_storage);
        self.rename_task_modal
            .pre_render(global_state, frame_storage);
        self.new_tag_modal.pre_render(global_state, frame_storage);
        self.search_box_depend_on
            .pre_render(global_state, frame_storage);

        frame_storage.add_keybind("⇅", "Navigate list", task_list.len() >= 2);
        frame_storage.add_keybind(
            KEYBIND_TASK_MARK_STARTED.to_string(),
            "Mark as started",
            frame_storage.selected_task_id.is_some(),
        );
        frame_storage.add_keybind(
            KEYBIND_TASK_MARK_DONE.to_string(),
            "Mark as done",
            frame_storage.selected_task_id.is_some(),
        );
        frame_storage.add_keybind(KEYBIND_TASK_NEW.to_string(), "New task", true);
        frame_storage.add_keybind(
            KEYBIND_TASK_DELETE.to_string(),
            "Delete task",
            frame_storage.selected_task_id.is_some(),
        );
        frame_storage.add_keybind(
            KEYBIND_TASK_ADD_TAG.to_string(),
            "Add tag",
            frame_storage.selected_task_id.is_some(),
        );
        frame_storage.add_keybind(
            KEYBIND_TASK_ADD_DEPENDENCY.to_string(),
            "Add dependency",
            frame_storage.selected_task_id.is_some(),
        );
        frame_storage.add_keybind(
            KEYBIND_TASK_RENAME.to_string(),
            "Rename",
            frame_storage.selected_task_id.is_some(),
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
            .map(|t| ListItem::new(self.task_to_span(state, t)))
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
        self.create_task_modal
            .render(frame, area, state, frame_storage);
        self.rename_task_modal
            .render(frame, area, state, frame_storage);
        self.new_tag_modal.render(frame, area, state, frame_storage);
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
        if self
            .create_task_modal
            .process_input(key, state, frame_storage)
            || self
                .rename_task_modal
                .process_input(key, state, frame_storage)
            || self.new_tag_modal.process_input(key, state, frame_storage)
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

        if self.create_task_modal.is_open() {
            // popup is open
            if key.code == KeyCode::Enter {
                if let Some(text) = self.create_task_modal.close() {
                    state.database.tasks.add_node(Task::create_now(text));

                    state.mark_database_dirty();
                }
                true
            } else {
                false
            }
        } else if self.rename_task_modal.is_open() {
            // popup is open
            if key.code == KeyCode::Enter {
                if let Some(text) = self.rename_task_modal.close() {
                    let selected_task = &mut state.database[&tasks[self.index].id];
                    selected_task.title = text;

                    state.mark_database_dirty();
                }
                true
            } else {
                false
            }
        } else if self.new_tag_modal.is_open() {
            // popup is open
            if key.code == KeyCode::Enter {
                if let Some(text) = self.new_tag_modal.close() {
                    let selected_task = &mut state.database[&tasks[self.index].id];
                    selected_task.tags.push(text);

                    state.mark_database_dirty();
                }
                true
            } else {
                false
            }
        } else if self.search_box_depend_on.is_open() {
            // popup is open
            if key.code == KeyCode::Enter {
                if let Some(selected_task_id) = self.search_box_depend_on.close() {
                    state
                        .database
                        .add_dependency(&tasks[self.index].id, &selected_task_id);

                    state.mark_database_dirty();
                }

                true
            } else {
                false
            }
        } else {
            // take our own input
            match (key.code, key.modifiers) {
                (KeyCode::Char(KEYBIND_TASK_MARK_STARTED), KeyModifiers::NONE) => {
                    let task = &mut state.database[&tasks[self.index].id];
                    if task.time_started.is_none() {
                        task.time_started = Some(
                            OffsetDateTime::now_local()
                                .unwrap_or_else(|_| OffsetDateTime::now_utc()),
                        );
                    } else {
                        task.time_started = None;
                    }

                    state.mark_database_dirty();

                    true
                }
                (KeyCode::Char(KEYBIND_TASK_MARK_DONE), KeyModifiers::NONE) => {
                    let task = &mut state.database[&tasks[self.index].id];
                    if task.time_completed.is_none() {
                        task.time_completed = Some(
                            OffsetDateTime::now_local()
                                .unwrap_or_else(|_| OffsetDateTime::now_utc()),
                        );
                    } else {
                        task.time_completed = None;
                    }

                    state.mark_database_dirty();

                    true
                }
                (KeyCode::Char(KEYBIND_TASK_NEW), KeyModifiers::NONE) => {
                    self.create_task_modal.open();
                    true
                }
                (KeyCode::Char(KEYBIND_TASK_RENAME), KeyModifiers::NONE) => {
                    self.rename_task_modal
                        .open_with_text(tasks[self.index].title.clone());
                    true
                }
                (KeyCode::Char(KEYBIND_TASK_DELETE), KeyModifiers::NONE) => {
                    if !tasks.is_empty() {
                        // delete
                        state.database.tasks.remove_node(
                            state
                                .database
                                .get_node_index(&tasks[self.index].id)
                                .unwrap(),
                        );

                        state.mark_database_dirty();
                    }

                    true
                }
                (KeyCode::Char(KEYBIND_TASK_ADD_TAG), KeyModifiers::NONE) => {
                    if !tasks.is_empty() {
                        // add tag to currently selected task
                        self.new_tag_modal.open();
                    }

                    true
                }
                (KeyCode::Char(KEYBIND_TASK_ADD_DEPENDENCY), KeyModifiers::NONE) => {
                    // link to other task
                    let selected = &tasks[self.index];
                    let existing_dependency_ids = state
                        .database
                        .get_dependencies(&selected.id)
                        .map(|x| x.id.clone())
                        .collect::<HashSet<_>>();
                    let candidate_tasks = tasks
                        .iter()
                        .filter(|t| t.id != selected.id)
                        .filter(|candidate| !existing_dependency_ids.contains(&candidate.id))
                        .map(|w| (w.id.clone(), w.title.clone()))
                        .collect();
                    self.search_box_depend_on.open(candidate_tasks);
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
                (KeyCode::PageUp, KeyModifiers::NONE) => {
                    self.index = self.index.saturating_sub(Self::SCROLL_PAGE_UP_DOWN);
                    true
                }
                (KeyCode::PageDown, KeyModifiers::NONE) => {
                    if !tasks.is_empty() && self.index != tasks.len() - 1 {
                        self.index += Self::SCROLL_PAGE_UP_DOWN;
                        self.index = self.index.min(tasks.len() - 1);
                    }
                    true
                }
                (KeyCode::Home, KeyModifiers::NONE) => {
                    self.index = 0;
                    true
                }
                (KeyCode::End, KeyModifiers::NONE) => {
                    if !tasks.is_empty() && self.index != tasks.len() - 1 {
                        self.index = tasks.len() - 1;
                    }
                    true
                }
                _ => false,
            }
        }
    }
}
