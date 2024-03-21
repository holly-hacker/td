use std::collections::HashSet;

use crossterm::event::KeyEvent;
use predicates::prelude::*;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
    Frame,
};
use td_lib::{
    database::{Task, TaskId},
    time::OffsetDateTime,
};

use super::task_search::TaskSearchBarComponent;
use crate::{
    keybinds::*,
    ui::{
        component_collection::{CollectionKey, ComponentCollection},
        constants::*,
        modal::*,
        AppState, Component, FrameLocalStorage,
    },
    utils::RectExt,
};

pub struct TaskList {
    focus: TaskListFocus,
    search_bar: TaskSearchBarComponent,
    modals: ComponentCollection,
    create_task_modal: CollectionKey<TextInputModal>,
    new_tag_modal: CollectionKey<TextInputModal>,
    rename_task_modal: CollectionKey<TextInputModal>,
    delete_task_modal: CollectionKey<ConfirmationModal>,
    edit_modal: CollectionKey<KeybindSelectModal>,
    search_box_depend_on: CollectionKey<ListSearchModal<TaskId>>,
}

enum TaskListFocus {
    SearchBar,
    Task(usize),
}

impl TaskList {
    const SCROLL_PAGE_UP_DOWN: usize = 32;

    pub fn new() -> Self {
        let mut modal_collection = ComponentCollection::default();
        Self {
            focus: TaskListFocus::Task(0),
            search_bar: TaskSearchBarComponent::default(),
            create_task_modal: modal_collection
                .insert(TextInputModal::new("Create new task".to_string())),
            new_tag_modal: modal_collection.insert(TextInputModal::new("Add new tag".to_string())),
            rename_task_modal: modal_collection
                .insert(TextInputModal::new("Rename task".to_string())),
            delete_task_modal: modal_collection.insert(
                ConfirmationModal::new("Do you want to delete this task?".to_string())
                    .with_title("Delete Task".to_string()),
            ),
            edit_modal: modal_collection.insert(KeybindSelectModal::new("Select an action".into())),
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
        if state.filter_search {
            tasks.retain(|t| self.search_bar.filter(t));
        }

        tasks
    }

    fn task_to_span(&self, state: &AppState, task: &Task) -> Line {
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

    fn set_focus(&mut self, value: TaskListFocus) {
        self.focus = value;
        match self.focus {
            TaskListFocus::SearchBar => self.search_bar.set_focus(true),
            TaskListFocus::Task(_) => self.search_bar.set_focus(false),
        }
    }
}

impl Component for TaskList {
    fn pre_render(&self, global_state: &AppState, frame_storage: &mut FrameLocalStorage) {
        match self.focus {
            TaskListFocus::SearchBar => {
                // select top-most task if possible. it's better than having none selected
                let task_list = self.get_task_list(global_state);
                frame_storage.selected_task_id = task_list.get(0).map(|x| x.id().clone());

                // NOTE: there should never be an open modal with the searchbar selected, but this
                // makes sure that they would work if it happened regardless.
                self.modals.pre_render(global_state, frame_storage);

                // show list navigation if there is at least 1 item to navigate to
                frame_storage
                    .register_keybind(KEYBIND_CONTROLS_LIST_NAV_EXT, !task_list.is_empty());
                frame_storage.register_keybind(KEYBIND_TASK_CLOSE_SEARCH, true);
            }
            TaskListFocus::Task(task_index) => {
                // store currently selected task in frame storage
                let task_list = self.get_task_list(global_state);
                frame_storage.selected_task_id = task_list.get(task_index).map(|x| x.id().clone());

                self.modals.pre_render(global_state, frame_storage);

                frame_storage.register_keybind(KEYBIND_CONTROLS_LIST_NAV_EXT, task_list.len() >= 2);

                let is_task_selected = frame_storage.selected_task_id.is_some();
                frame_storage.register_keybind(KEYBIND_TASK_MARK_STARTED, is_task_selected);
                frame_storage.register_keybind(KEYBIND_TASK_MARK_DONE, is_task_selected);
                frame_storage.register_keybind(KEYBIND_TASK_NEW, true);
                frame_storage.register_keybind(KEYBIND_TASK_DELETE, is_task_selected);
                frame_storage.register_keybind(KEYBIND_TASK_ADD_TAG, is_task_selected);
                frame_storage.register_keybind(KEYBIND_TASK_ADD_DEPENDENCY, is_task_selected);
                frame_storage.register_keybind(KEYBIND_TASK_RENAME, is_task_selected);
                frame_storage.register_keybind(KEYBIND_TASK_EDIT, is_task_selected);
                frame_storage.register_keybind(KEYBIND_TASK_TOGGLE_SEARCH, true);
            }
        }
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &AppState,
        frame_storage: &crate::ui::FrameLocalStorage,
    ) {
        let task_list = self.get_task_list(state);

        let list_area;

        if state.filter_search {
            list_area = area.skip_y(1);

            let search_area = area.take_y(1);
            self.search_bar
                .render(frame, search_area, state, frame_storage);
        } else {
            list_area = area;
        }

        // render the list
        let list_items = task_list
            .iter()
            .map(|t| ListItem::new(self.task_to_span(state, t)))
            .collect::<Vec<_>>();
        let list = List::new(list_items)
            .highlight_style(if matches!(self.focus, TaskListFocus::Task(_)) {
                LIST_HIGHLIGHT_STYLE
            } else {
                LIST_HIGHLIGHT_STYLE_DISABLED
            })
            .style(LIST_STYLE);
        let mut list_state = ListState::default();
        if let TaskListFocus::Task(task_index) = self.focus {
            list_state.select((!task_list.is_empty()).then_some(task_index));
        } else {
            list_state.select((!task_list.is_empty()).then_some(0));
        }
        frame.render_stateful_widget(list, list_area, &mut list_state);

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

        // safety checks
        if let TaskListFocus::Task(task_index) = &mut self.focus {
            if !tasks.is_empty() {
                *task_index = (*task_index).clamp(0, tasks.len() - 1);
            }
        }

        match self.focus {
            TaskListFocus::SearchBar => {
                if KEYBIND_CONTROLS_LIST_NAV_EXT.get_match(key) == Some(UpDownExtendedKey::Down) {
                    self.set_focus(TaskListFocus::Task(0));
                    true
                } else if KEYBIND_TASK_CLOSE_SEARCH.is_match(key) {
                    state.filter_search = false;
                    self.set_focus(TaskListFocus::Task(0));
                    true
                } else {
                    self.search_bar.process_input(key, state, frame_storage)
                }
            }
            TaskListFocus::Task(task_index) => {
                if self.handle_modals(key, state, &tasks, task_index) {
                    return true;
                }

                // take our own input
                // start by checking actions that require a task to present
                let handled_by_task = if !tasks.is_empty() {
                    if KEYBIND_TASK_MARK_STARTED.is_match(key) {
                        state.database.modify(|db| {
                            let task = &mut db[tasks[task_index].id()];
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
                            let task = &mut db[tasks[task_index].id()];
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
                    } else if KEYBIND_TASK_RENAME.is_match(key) {
                        self.modals[self.rename_task_modal]
                            .open_with_text(tasks[task_index].title.clone());
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
                        let modal = &mut self.modals[self.search_box_depend_on];
                        Self::open_add_dependency_dialog(modal, state, task_index, &tasks);
                        true
                    } else if KEYBIND_TASK_EDIT.is_match(key) {
                        self.modals[self.edit_modal].open(vec![
                            KEYBIND_TASK_RENAME.clone(),
                            KEYBIND_TASK_DELETE.clone(),
                            KEYBIND_TASK_ADD_DEPENDENCY.clone(),
                            KEYBIND_TASK_ADD_TAG.clone(),
                        ]);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                // if the input wasn't handled by a task, check the other keybinds
                handled_by_task
                    || if KEYBIND_TASK_NEW.is_match(key) {
                        self.modals[self.create_task_modal].open();
                        true
                    } else if KEYBIND_TASK_TOGGLE_SEARCH.is_match(key) {
                        state.filter_search = !state.filter_search;

                        // if we are turning *on* search, focus the search bar
                        if state.filter_search {
                            self.set_focus(TaskListFocus::SearchBar);
                        }

                        true
                    } else if let Some(key) = KEYBIND_CONTROLS_LIST_NAV_EXT.get_match(key) {
                        // handle kb navigation

                        if key == UpDownExtendedKey::Up && task_index == 0 && state.filter_search {
                            self.set_focus(TaskListFocus::SearchBar);
                            return true;
                        }

                        let TaskListFocus::Task(task_index) = &mut self.focus else {unreachable!();};

                        match key {
                            UpDownExtendedKey::Up => {
                                *task_index = task_index.saturating_sub(1);
                                true
                            }
                            UpDownExtendedKey::Down => {
                                if !tasks.is_empty() && *task_index != tasks.len() - 1 {
                                    *task_index += 1;
                                }
                                true
                            }
                            UpDownExtendedKey::PageUp => {
                                *task_index = task_index.saturating_sub(Self::SCROLL_PAGE_UP_DOWN);
                                true
                            }
                            UpDownExtendedKey::PageDown => {
                                if !tasks.is_empty() && *task_index != tasks.len() - 1 {
                                    *task_index += Self::SCROLL_PAGE_UP_DOWN;
                                    *task_index = (*task_index).min(tasks.len() - 1);
                                }
                                true
                            }
                            UpDownExtendedKey::Home => {
                                *task_index = 0;
                                true
                            }
                            UpDownExtendedKey::End => {
                                if !tasks.is_empty() && *task_index != tasks.len() - 1 {
                                    *task_index = tasks.len() - 1;
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
}

impl TaskList {
    fn handle_modals(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
        tasks: &[Task],
        task_index: usize,
    ) -> bool {
        if self.modals[self.edit_modal].is_open() {
            if let Some(selected) = self.modals[self.edit_modal].take_selected_keybind() {
                match selected {
                    _ if selected == *KEYBIND_TASK_RENAME => {
                        self.modals[self.rename_task_modal]
                            .open_with_text(tasks[task_index].title.clone());
                        return true;
                    }
                    _ if selected == *KEYBIND_TASK_DELETE => {
                        self.modals[self.delete_task_modal].open(true);
                        return true;
                    }
                    _ if selected == *KEYBIND_TASK_ADD_DEPENDENCY => {
                        let modal = &mut self.modals[self.search_box_depend_on];
                        Self::open_add_dependency_dialog(modal, state, task_index, tasks);
                        return true;
                    }
                    _ if selected == *KEYBIND_TASK_ADD_TAG => {
                        if !tasks.is_empty() {
                            // add tag to currently selected task
                            self.modals[self.new_tag_modal].open();
                        }
                        return true;
                    }
                    _ => (),
                }
            }
            // always return true because the modal should be blocking input propagation but it
            // can't since it blocks us from checking the modal result. thus, we block here.
            true
        } else if self.modals[self.create_task_modal].is_open() {
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
                        let selected_task = &mut db[tasks[task_index].id()];
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
                        .modify(|x| x.remove_task(tasks[task_index].id()));
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
                        let selected_task = &mut db[tasks[task_index].id()];
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
                        .modify(|x| x.add_dependency(tasks[task_index].id(), &selected_task_id));
                }

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn open_add_dependency_dialog(
        modal: &mut ListSearchModal<TaskId>,
        state: &AppState,
        task_index: usize,
        tasks: &[Task],
    ) {
        // link to other task
        let selected = &tasks[task_index];
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
        modal.open(candidate_tasks);
    }
}
