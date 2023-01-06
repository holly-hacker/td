use std::{error::Error, io::Stdout, path::PathBuf};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use td_lib::{
    database::{Database, DatabaseInfo},
    errors::DatabaseReadError,
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
            terminal.draw(|f| root_component.render(f, f.size(), self))?;

            if let Event::Key(key) = event::read()? {
                let handled = root_component.update(key, self);
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

pub trait Component {
    /// Render the component and its children to the given area.
    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, state: &AppState);

    /// Update state based in a key event. Returns whether the key event is handled by this
    /// component or one of its children.
    fn update(&mut self, key: KeyEvent, state: &mut AppState) -> bool;

    // TODO: may need to split update into input+update
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
    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, state: &AppState) {
        self.tabs.render(frame, area, state);
    }

    fn update(&mut self, key: KeyEvent, state: &mut AppState) -> bool {
        self.tabs.update(key, state)
    }
}
