use std::{borrow::Cow, error::Error, io::Stdout, path::PathBuf};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use td_lib::{
    database::{Database, DatabaseInfo},
    errors::DatabaseReadError,
    petgraph::stable_graph::NodeIndex,
};
use tui::{backend::CrosstermBackend, layout::Rect, Frame, Terminal};

use self::{keybind_list::KeybindList, tab_layout::TabLayout, tasks::task_list::BasicTaskList};
use crate::utils::{wrap_spans, RectExt};

mod constants;
mod input;
mod keybind_list;
mod modal;
mod tab_layout;
mod task_info;
mod tasks;

pub struct AppState {
    pub database: Database,
    pub path: PathBuf,
}

impl AppState {
    pub fn create(path: PathBuf) -> Result<Self, DatabaseReadError> {
        let db_info = if !path.exists() {
            println!("The given database file ({path:?}) does not exist, creating a new one.");

            let db_info = DatabaseInfo::default();
            db_info.write(&path)?;
            db_info
        } else {
            DatabaseInfo::read(&path)?
        };

        let database = db_info.try_into()?;

        Ok(Self { database, path })
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
}

/// Global storage for the current frame. Can be populated during [Component::pre_render] and read
/// during [Component::render] and [Component::process_input].
#[derive(Default)]
pub struct FrameLocalStorage {
    /// A map of keybind to action for the currently rendering frame
    current_keybinds: Vec<(Cow<'static, str>, Cow<'static, str>, bool)>,
    keybinds_locked: bool,

    /// The currently selected/focused task
    selected_task_index: Option<NodeIndex>,
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

pub trait Component {
    /// Executed before the render pass. Can be used to collect information that is required in the
    /// render pass. This is guaranteed to run once before each [Component::render] call.
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

struct LayoutRoot {
    tabs: TabLayout,
}

impl LayoutRoot {
    fn new() -> Self {
        Self {
            tabs: TabLayout::new([
                (
                    "Newest",
                    Box::new(BasicTaskList::new(true)) as Box<dyn Component>,
                ),
                (
                    "Oldest",
                    Box::new(BasicTaskList::new(false)) as Box<dyn Component>,
                ),
            ]),
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
