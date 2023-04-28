use std::{borrow::Cow, collections::HashSet, error::Error, io::Stdout, path::PathBuf};

use crossterm::event::{self, Event, KeyEvent};
use downcast_rs::{impl_downcast, Downcast};
use predicates::{
    prelude::{predicate, PredicateBooleanExt},
    BoxPredicate, PredicateBoxExt,
};
use td_lib::{
    database::{database_file::DatabaseFile, Database, Task, TaskId},
    errors::DatabaseReadError,
};
use td_util::undo::UndoWrapper;
use tui::{backend::CrosstermBackend, layout::Rect, Frame, Terminal};

use self::{
    keybind_list::KeybindList, modal::ConfirmationModal, tab_layout::TabLayout, tasks::TaskPage,
};
use crate::{
    keybinds::*,
    utils::{wrap_spans, MapPredicate, RectExt},
};

mod component_collection;
mod constants;
mod dirty_indicator;
mod input;
mod keybind_list;
mod modal;
mod tab_layout;
mod tasks;

#[cfg_attr(test, derive(Default))]
pub struct AppState {
    pub database: UndoWrapper<Database>,
    pub path: PathBuf,

    should_exit: bool,

    pub sort_oldest_first: bool,
    pub filter_completed: bool,
    pub filter_unactionable: bool,
    pub filter_search: bool,
}

impl AppState {
    pub fn create(path: PathBuf) -> Result<Self, DatabaseReadError> {
        let db_info = if !path.exists() {
            println!("The given database file ({path:?}) does not exist, creating a new one.");

            let db_info = DatabaseFile::default();
            db_info.write(&path)?;
            db_info
        } else {
            DatabaseFile::read(&path)?
        };

        let mut database: UndoWrapper<Database> = UndoWrapper::new(db_info.try_into()?);
        database.mark_clean();

        Ok(Self {
            database,
            path,
            should_exit: false,
            sort_oldest_first: false,
            filter_completed: true,
            filter_unactionable: false,
            filter_search: false,
        })
    }

    pub fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut root_component = LayoutRoot::new();

        loop {
            let mut frame_storage = FrameLocalStorage::default();
            root_component.pre_render(self, &mut frame_storage);

            terminal.draw(|f| root_component.render(f, f.size(), self, &frame_storage))?;

            if let Event::Key(key) = event::read()? {
                _ = root_component.process_input(key, self, &frame_storage);

                if self.should_exit {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn request_exit(&mut self) {
        self.should_exit = true;
    }

    /// Saves the database to disk and marks it as clean.
    pub fn save(&mut self) {
        // TODO: error handling. show popup on failure to save?
        let db_info: DatabaseFile = (&*self.database).into();
        db_info.write(&self.path).unwrap();
        self.database.mark_clean();
    }

    pub fn get_task_filter_predicate(&self) -> BoxPredicate<Task> {
        let mut predicate = predicate::always().boxed();

        if self.filter_completed {
            predicate = predicate
                .and(predicate::function(|x: &Task| x.time_completed.is_none()))
                .boxed();
        }

        if self.filter_unactionable {
            let tasks_with_uncompleted_dependencies = self
                .database
                .get_all_tasks()
                .filter(|t| {
                    self.database
                        .get_dependencies(t.id())
                        .any(|dep| dep.time_completed.is_none())
                })
                .map(|t| t.id().clone())
                .collect::<HashSet<_>>();

            let has_uncompleted_dependencies =
                predicate::in_hash(tasks_with_uncompleted_dependencies);

            let has_uncompleted_dependencies =
                MapPredicate::new(has_uncompleted_dependencies, |task: &Task| task.id());

            predicate = predicate.and(has_uncompleted_dependencies.not()).boxed();
        }

        predicate
    }
}

/// Global storage for the current frame. Can be populated during [Component::pre_render] and read
/// during [Component::render] and [Component::process_input].
#[derive(Default)]
pub struct FrameLocalStorage {
    /// A map of keybind to action for the currently rendering frame
    current_keybinds: Vec<(Cow<'static, str>, Cow<'static, str>, bool)>,
    keybinds_locked: bool,

    /// The currently selected/focused task
    selected_task_id: Option<TaskId>,
}

impl FrameLocalStorage {
    /// Registers a keybind to be shown with [KeybindList].
    pub fn register_keybind(&mut self, keybind: &dyn Keybind, enabled: bool) {
        if self.keybinds_locked {
            return;
        }

        if let Some(desc) = keybind.description() {
            let char = keybind.key_hint();
            debug_assert_eq!(
                self.current_keybinds.iter().find(|x| x.0 == *char),
                None,
                "duplicate keybind: {char} (added as '{desc}')"
            );
            self.current_keybinds
                .push((char.clone(), desc.clone(), enabled));
        }
    }

    /// Disallows more keybinds to be added.
    pub fn lock_keybinds(&mut self) {
        self.keybinds_locked = true;
    }
}

pub trait Component: Downcast {
    /// Executed before the render pass. Can be used to collect information that is required in the
    /// render pass and to register keybind hints. This is guaranteed to run once before each
    /// [Component::render] call.
    fn pre_render(&self, _global_state: &AppState, _frame_storage: &mut FrameLocalStorage) {}

    /// Render the component and its children to the given area.
    fn render(
        &self,
        frame: &mut Frame<CrosstermBackend<Stdout>>,
        area: Rect,
        state: &AppState,
        frame_storage: &FrameLocalStorage,
    );

    /// Update state based in a key event. Returns whether the key event is handled by this
    /// component or one of its children and no other components should process them.
    fn process_input(
        &mut self,
        _key: KeyEvent,
        _state: &mut AppState,
        _frame_storage: &FrameLocalStorage,
    ) -> bool {
        false
    }
}

impl_downcast!(Component);

struct LayoutRoot {
    tabs: TabLayout,
    save_unsaved_confirmation: ConfirmationModal,
}

impl LayoutRoot {
    fn new() -> Self {
        Self {
            tabs: TabLayout::new([("Tasks", Box::new(TaskPage::new()) as Box<dyn Component>)]),
            save_unsaved_confirmation: ConfirmationModal::new(
                "There are unsaved changes. Do you want to save before quitting?".into(),
            )
            .with_title("Save before quitting?".into()),
        }
    }
}

impl Component for LayoutRoot {
    fn pre_render(&self, state: &AppState, frame_storage: &mut FrameLocalStorage) {
        self.save_unsaved_confirmation
            .pre_render(state, frame_storage);
        self.tabs.pre_render(state, frame_storage);

        frame_storage.register_keybind(KEYBIND_SAVE, state.database.is_dirty());
        frame_storage.register_keybind(KEYBIND_UNDO, state.database.undo_count() > 0);
        frame_storage.register_keybind(KEYBIND_REDO, state.database.redo_count() > 0);
        frame_storage.register_keybind(KEYBIND_QUIT, true);
        frame_storage.register_keybind(KEYBIND_QUIT_ALT, true);
    }

    fn render(
        &self,
        frame: &mut Frame<CrosstermBackend<Stdout>>,
        area: Rect,
        state: &AppState,
        frame_storage: &FrameLocalStorage,
    ) {
        let height = wrap_spans(KeybindList::get_spans(frame_storage), area.width).len() as u16;

        let (area_tabs, area_keybinds) = area.split_last_y(height);
        self.tabs.render(frame, area_tabs, state, frame_storage);

        KeybindList.render(frame, area_keybinds, state, frame_storage);

        self.save_unsaved_confirmation
            .render(frame, area, state, frame_storage);
    }

    fn process_input(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
        frame_storage: &FrameLocalStorage,
    ) -> bool {
        if self
            .save_unsaved_confirmation
            .process_input(key, state, frame_storage)
        {
            return true;
        }

        if self.save_unsaved_confirmation.is_open() {
            if KEYBIND_MODAL_SUBMIT.is_match(key) {
                if self.save_unsaved_confirmation.close() {
                    state.save();
                }
                state.request_exit();
                return true;
            } else {
                return false;
            }
        }

        if self.tabs.process_input(key, state, frame_storage) {
            return true;
        }

        if KEYBIND_SAVE.is_match(key) {
            state.save();
            true
        } else if KEYBIND_UNDO.is_match(key) && state.database.undo_count() > 0 {
            state.database.undo();
            true
        } else if KEYBIND_REDO.is_match(key) && state.database.redo_count() > 0 {
            state.database.redo();
            true
        } else if KEYBIND_QUIT.is_match(key) || KEYBIND_QUIT_ALT.is_match(key) {
            if state.database.is_dirty() {
                self.save_unsaved_confirmation.open(true);
            } else {
                state.request_exit();
            }
            true
        } else {
            false
        }
    }
}
