use std::{borrow::Cow, error::Error, io::Stdout, path::PathBuf};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use downcast_rs::{impl_downcast, Downcast};
use predicates::{
    prelude::{predicate, PredicateBooleanExt},
    BoxPredicate, PredicateBoxExt,
};
use td_lib::{
    database::{database_file::DatabaseFile, Database, Task, TaskId},
    errors::DatabaseReadError,
};
use tui::{backend::CrosstermBackend, layout::Rect, Frame, Terminal};

use self::{keybind_list::KeybindList, tab_layout::TabLayout, tasks::TaskPage};
use crate::utils::{wrap_spans, RectExt};

mod constants;
mod input;
mod keybind_list;
mod modal;
mod tab_layout;
mod tasks;

#[cfg_attr(test, derive(Default))]
pub struct AppState {
    pub database: Database,
    pub path: PathBuf,

    pub sort_oldest_first: bool,
    pub filter_completed: bool,
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

        let database = db_info.try_into()?;

        Ok(Self {
            database,
            path,
            sort_oldest_first: false,
            filter_completed: true,
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
            frame_storage.add_keybind("⎋, q", "Quit", true);

            terminal.draw(|f| root_component.render(f, f.size(), self, &frame_storage))?;

            if let Event::Key(key) = event::read()? {
                let handled = root_component.process_input(key, self, &frame_storage);
                if !handled {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break
                        }
                        _ => (),
                    }
                }
            }
        }

        Ok(())
    }

    pub fn mark_database_dirty(&mut self) {
        // TODO: error handling. show popup on failure to save?
        // TODO: don't immediately save, store dirty flag instead.
        let db_info: DatabaseFile = (&self.database).into();
        db_info.write(&self.path).unwrap();
    }

    pub fn get_task_filter_predicate(&self) -> BoxPredicate<Task> {
        let mut predicate = predicate::always().boxed();

        if self.filter_completed {
            predicate = predicate
                .and(predicate::function(|x: &Task| x.time_completed.is_none()))
                .boxed();
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
    pub fn add_keybind(
        &mut self,
        keybind: impl Into<Cow<'static, str>>,
        description: impl Into<Cow<'static, str>>,
        enabled: bool,
    ) {
        if self.keybinds_locked {
            return;
        }

        let (keybind, description) = (keybind.into(), description.into());
        debug_assert_eq!(
            self.current_keybinds.iter().find(|x| x.0 == keybind),
            None,
            "duplicate keybind: {keybind} (added as '{description}')"
        );
        self.current_keybinds.push((keybind, description, enabled));
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
    fn pre_render(&self, global_state: &AppState, frame_storage: &mut FrameLocalStorage);

    /// Render the component and its children to the given area.
    fn render(
        &self,
        frame: &mut Frame<CrosstermBackend<Stdout>>,
        area: Rect,
        state: &AppState,
        frame_storage: &FrameLocalStorage,
    );

    /// Update state based in a key event. Returns whether the key event is handled by this
    /// component or one of its children.
    fn process_input(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
        frame_storage: &FrameLocalStorage,
    ) -> bool;
}

impl_downcast!(Component);

struct LayoutRoot {
    tabs: TabLayout,
}

impl LayoutRoot {
    fn new() -> Self {
        Self {
            tabs: TabLayout::new([("Tasks", Box::new(TaskPage::new()) as Box<dyn Component>)]),
        }
    }
}

impl Component for LayoutRoot {
    fn pre_render(&self, global_state: &AppState, storage: &mut FrameLocalStorage) {
        self.tabs.pre_render(global_state, storage);
    }

    fn render(
        &self,
        frame: &mut Frame<CrosstermBackend<Stdout>>,
        area: Rect,
        state: &AppState,
        frame_storage: &FrameLocalStorage,
    ) {
        let height = wrap_spans(KeybindList::get_spans(frame_storage), area.width).len() as u16;

        let area_tabs = area.skip_last_y(height);
        let area_keybinds = area.take_last_y(height);
        self.tabs.render(frame, area_tabs, state, frame_storage);

        KeybindList.render(frame, area_keybinds, state, frame_storage);
    }

    fn process_input(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
        frame_storage: &FrameLocalStorage,
    ) -> bool {
        self.tabs.process_input(key, state, frame_storage)
    }
}
