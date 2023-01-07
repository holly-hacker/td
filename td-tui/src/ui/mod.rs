use std::{error::Error, io::Stdout, path::PathBuf};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use td_lib::{
    database::{Database, DatabaseInfo},
    errors::DatabaseReadError,
    petgraph::stable_graph::NodeIndex,
};
use tui::{backend::CrosstermBackend, layout::Rect, Frame, Terminal};

use self::{tab_layout::TabLayout, tasks::task_list::BasicTaskList};

mod constants;
mod input;
mod modal;
mod tab_layout;
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
    /// The currently selected/focused task
    pub selected_task_index: Option<NodeIndex>,
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
        self.tabs.render(frame, area, state, frame_storage);
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
