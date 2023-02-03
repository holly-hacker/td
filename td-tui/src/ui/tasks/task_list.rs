use std::{collections::HashSet, io::Stdout};

use crossterm::event::KeyEvent;
use predicates::prelude::*;
use td_lib::{
    database::{Task, TaskId},
    time::OffsetDateTime,
};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    text::{Span, Spans},
    widgets::{List, ListItem, ListState},
    Frame,
};

use crate::{
    keybinds::*,
    ui::{
        component_collection::{CollectionKey, ComponentCollection},
        constants::*,
        modal::*,
        AppState, Component, FrameLocalStorage,
    },
};

pub struct TaskList {
    index: usize,
    modals: ComponentCollection,
    create_task_modal: CollectionKey<TextInputModal>,
    new_tag_modal: CollectionKey<TextInputModal>,
    rename_task_modal: CollectionKey<TextInputModal>,
    delete_task_modal: CollectionKey<ConfirmationModal>,
    search_box_depend_on: CollectionKey<ListSearchModal<TaskId>>,
}

impl TaskList {
    const SCROLL_PAGE_UP_DOWN: usize = 32;

    pub fn new() -> Self {
        let mut modal_collection = ComponentCollection::default();
        Self {
            index: 0,
            create_task_modal: modal_collection
                .insert(TextInputModal::new("Create new task".to_string())),
            new_tag_modal: modal_collection.insert(TextInputModal::new("Add new tag".to_string())),
            rename_task_modal: modal_collection
                .insert(TextInputModal::new("Rename task".to_string())),
            delete_task_modal: modal_collection.insert(
                ConfirmationModal::new("Do you want to delete this task?".to_string())
                    .with_title("Delete Task".to_string()),
            ),
            search_box_depend_on: modal_collection.insert(ListSearchModal::new(
                "Choose which task to depend on".to_string(),
            )),
            modals: modal_collection,
        }
    }

    fn get_task_list(&self, state: &AppState) -> Vec<Task> {
        let mut tasks = state.database.get_all_tasks().cloned().collect::<Vec<_>>();

        // sort
        tasks.sort_by(|a, b| a.time_created.cmp(&b.time_created));
        if !state.sort_oldest_first {
            tasks.reverse();
        }

        // filter
        tasks.retain(|x| state.get_task_filter_predicate().eval(x));

        tasks
    }

    fn task_to_span(&self, state: &AppState, task: &Task) -> Spans {
        let mut spans = vec![];

        let dependents_count = state.database.get_inverse_dependencies(task.id()).count();
        if dependents_count > 0 {
            spans.push(Span::styled(
                format!("{:>2}⤣", dependents_count.to_string()),
                FG_GREEN.patch(BOLD),
            ));
        }

        let unfullfilled_dependency_count = state
            .database
            .get_dependencies(task.id())
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

impl Component for TaskList {
    fn pre_render(&self, global_state: &AppState, frame_storage: &mut FrameLocalStorage) {
        // store currently selected task in frame storage
        let task_list = self.get_task_list(global_state);
        frame_storage.selected_task_id = task_list.get(self.index).map(|x| x.id().clone());

        self.modals.pre_render(global_state, frame_storage);

        frame_storage.register_keybind(KEYBIND_CONTROLS_LIST_NAV_EXT, task_list.len() >= 2);
        frame_storage.register_keybind(
            KEYBIND_TASK_MARK_STARTED,
            frame_storage.selected_task_id.is_some(),
        );
        frame_storage.register_keybind(
            KEYBIND_TASK_MARK_DONE,
            frame_storage.selected_task_id.is_some(),
        );
        frame_storage.register_keybind(KEYBIND_TASK_NEW, true);
        frame_storage.register_keybind(
            KEYBIND_TASK_DELETE,
            frame_storage.selected_task_id.is_some(),
        );
        frame_storage.register_keybind(
            KEYBIND_TASK_ADD_TAG,
            frame_storage.selected_task_id.is_some(),
        );
        frame_storage.register_keybind(
            KEYBIND_TASK_ADD_DEPENDENCY,
            frame_storage.selected_task_id.is_some(),
        );
        frame_storage.register_keybind(
            KEYBIND_TASK_RENAME,
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
        let task_list = self.get_task_list(state);

        // render the list
        let list_items = task_list
            .iter()
            .map(|t| ListItem::new(self.task_to_span(state, t)))
            .collect::<Vec<_>>();
        let list = List::new(list_items)
            .highlight_style(LIST_HIGHLIGHT_STYLE)
            .style(LIST_STYLE);
        let mut list_state = ListState::default();
        list_state.select((!task_list.is_empty()).then_some(self.index));
        frame.render_stateful_widget(list, area, &mut list_state);

        // if needed, render popups
        self.modals
            .render(frame, frame.size(), state, frame_storage);
    }

    fn process_input(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
        frame_storage: &FrameLocalStorage,
    ) -> bool {
        // check modals
        if self.modals.process_input(key, state, frame_storage) {
            return true;
        }

        let tasks = self.get_task_list(state);

        if !tasks.is_empty() {
            self.index = self.index.clamp(0, tasks.len() - 1);
        }

        if self.modals[self.create_task_modal].is_open() {
            // popup is open
            if KEYBIND_MODAL_SUBMIT.is_match(key) {
                if let Some(text) = self.modals[self.create_task_modal].close() {
                    state
                        .database
                        .modify(|x| x.add_task(Task::create_now(text)));
                }
                true
            } else {
                false
            }
        } else if self.modals[self.rename_task_modal].is_open() {
            // popup is open
            if KEYBIND_MODAL_SUBMIT.is_match(key) {
                if let Some(text) = self.modals[self.rename_task_modal].close() {
                    state.database.modify(|db| {
                        let selected_task = &mut db[tasks[self.index].id()];
                        selected_task.title = text;
                    });
                }
                true
            } else {
                false
            }
        } else if self.modals[self.delete_task_modal].is_open() {
            // popup is open
            if KEYBIND_MODAL_SUBMIT.is_match(key) {
                if self.modals[self.delete_task_modal].close() && !tasks.is_empty() {
                    // delete
                    state
                        .database
                        .modify(|x| x.remove_task(tasks[self.index].id()));
                }
                true
            } else {
                false
            }
        } else if self.modals[self.new_tag_modal].is_open() {
            // popup is open
            if KEYBIND_MODAL_SUBMIT.is_match(key) {
                if let Some(text) = self.modals[self.new_tag_modal].close() {
                    state.database.modify(|db| {
                        let selected_task = &mut db[tasks[self.index].id()];
                        selected_task.tags.push(text);
                    });
                }
                true
            } else {
                false
            }
        } else if self.modals[self.search_box_depend_on].is_open() {
            // popup is open
            if KEYBIND_MODAL_SUBMIT.is_match(key) {
                if let Some(selected_task_id) = self.modals[self.search_box_depend_on].close() {
                    state
                        .database
                        .modify(|x| x.add_dependency(tasks[self.index].id(), &selected_task_id));
                }

                true
            } else {
                false
            }
        } else {
            // take our own input
            if KEYBIND_TASK_MARK_STARTED.is_match(key) {
                state.database.modify(|db| {
                    let task = &mut db[tasks[self.index].id()];
                    if task.time_started.is_none() {
                        task.time_started = Some(
                            OffsetDateTime::now_local()
                                .unwrap_or_else(|_| OffsetDateTime::now_utc()),
                        );
                    } else {
                        task.time_started = None;
                    }
                });

                true
            } else if KEYBIND_TASK_MARK_DONE.is_match(key) {
                state.database.modify(|db| {
                    let task = &mut db[tasks[self.index].id()];
                    if task.time_completed.is_none() {
                        task.time_completed = Some(
                            OffsetDateTime::now_local()
                                .unwrap_or_else(|_| OffsetDateTime::now_utc()),
                        );
                    } else {
                        task.time_completed = None;
                    }
                });

                true
            } else if KEYBIND_TASK_NEW.is_match(key) {
                self.modals[self.create_task_modal].open();
                true
            } else if KEYBIND_TASK_RENAME.is_match(key) {
                self.modals[self.rename_task_modal].open_with_text(tasks[self.index].title.clone());
                true
            } else if KEYBIND_TASK_DELETE.is_match(key) {
                self.modals[self.delete_task_modal].open(true);

                true
            } else if KEYBIND_TASK_ADD_TAG.is_match(key) {
                if !tasks.is_empty() {
                    // add tag to currently selected task
                    self.modals[self.new_tag_modal].open();
                }

                true
            } else if KEYBIND_TASK_ADD_DEPENDENCY.is_match(key) {
                // link to other task
                let selected = &tasks[self.index];
                let existing_dependency_ids = state
                    .database
                    .get_dependencies(selected.id())
                    .map(|x| x.id().clone())
                    .collect::<HashSet<_>>();
                let candidate_tasks = tasks
                    .iter()
                    .filter(|t| t.id() != selected.id())
                    .filter(|candidate| !existing_dependency_ids.contains(candidate.id()))
                    .map(|w| (w.id().clone(), w.title.clone()))
                    .collect();
                self.modals[self.search_box_depend_on].open(candidate_tasks);
                true
            } else if let Some(key) = KEYBIND_CONTROLS_LIST_NAV_EXT.get_match(key) {
                match key {
                    UpDownExtendedKey::Up => {
                        self.index = self.index.saturating_sub(1);
                        true
                    }
                    UpDownExtendedKey::Down => {
                        if !tasks.is_empty() && self.index != tasks.len() - 1 {
                            self.index += 1;
                        }
                        true
                    }
                    UpDownExtendedKey::PageUp => {
                        self.index = self.index.saturating_sub(Self::SCROLL_PAGE_UP_DOWN);
                        true
                    }
                    UpDownExtendedKey::PageDown => {
                        if !tasks.is_empty() && self.index != tasks.len() - 1 {
                            self.index += Self::SCROLL_PAGE_UP_DOWN;
                            self.index = self.index.min(tasks.len() - 1);
                        }
                        true
                    }
                    UpDownExtendedKey::Home => {
                        self.index = 0;
                        true
                    }
                    UpDownExtendedKey::End => {
                        if !tasks.is_empty() && self.index != tasks.len() - 1 {
                            self.index = tasks.len() - 1;
                        }
                        true
                    }
                }
            } else {
                false
            }
        }
    }
}
