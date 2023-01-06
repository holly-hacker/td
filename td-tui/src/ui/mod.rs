use std::{error::Error, io::Stdout, path::PathBuf, time::SystemTime};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use td_lib::{
    database::{Database, DatabaseInfo, Task, TaskDependency},
    errors::DatabaseReadError,
    petgraph::graph::NodeIndex,
};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

use self::{
    constants::{LIST_HIGHLIGHT_STYLE, LIST_STYLE, STANDARD_STYLE_FG_WHITE},
    modal::{list_search::ListSearchModal, text_input::TextInputModal},
    tab_layout::TabLayout,
};

mod constants;
mod input;
mod modal;
mod tab_layout;

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
                        KeyCode::Char('s') => {
                            // todo: save
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
                    "Tasks",
                    Box::new(BasicTaskList::new(false)) as Box<dyn Component>,
                ),
                (
                    "Tasks (rev)",
                    Box::new(BasicTaskList::new(true)) as Box<dyn Component>,
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

struct BasicTaskList {
    index: usize,
    task_popup: TextInputModal,
    search_box_depend_on: ListSearchModal<NodeIndex>,
    reverse: bool,
}

impl BasicTaskList {
    fn new(reverse: bool) -> Self {
        Self {
            index: 0,
            task_popup: TextInputModal::new("Enter new task".to_string()),
            search_box_depend_on: ListSearchModal::new(
                "Choose which task to depend on".to_string(),
            ),
            reverse,
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
        if self.reverse {
            tasks.reverse();
        }

        tasks
    }
}

impl Component for BasicTaskList {
    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, state: &AppState) {
        let tasks = self.get_task_list(state);

        // render the list
        let block = Block::default()
            .title(if !self.reverse {
                "Basic Task List"
            } else {
                "Basic Task List (reversed)"
            })
            .borders(Borders::ALL)
            .border_style(STANDARD_STYLE_FG_WHITE)
            .border_type(BorderType::Rounded);

        let list_items = tasks
            .iter()
            .map(|t| ListItem::new(t.1.title.clone()))
            .collect::<Vec<_>>();
        let list = List::new(list_items)
            .block(block)
            .highlight_style(LIST_HIGHLIGHT_STYLE)
            .style(LIST_STYLE);
        let mut list_state = ListState::default();
        list_state.select((!tasks.is_empty()).then_some(self.index));
        frame.render_stateful_widget(list, area, &mut list_state);

        // if needed, render popups
        self.task_popup.render(frame, area, state);
        self.search_box_depend_on.render(frame, area, state);
    }

    fn update(&mut self, key: KeyEvent, state: &mut AppState) -> bool {
        if self.task_popup.update(key, state) {
            return true;
        }
        if self.search_box_depend_on.update(key, state) {
            return true;
        }

        let tasks = self.get_task_list(state);

        if !tasks.is_empty() {
            self.index = self.index.clamp(0, tasks.len() - 1);
        }

        if self.task_popup.is_open() {
            // popup is open
            match key.code {
                KeyCode::Enter => {
                    if let Some(text) = self.task_popup.close() {
                        let task = Task {
                            title: text,
                            time_created: SystemTime::now(),
                        };
                        state.database.tasks.add_node(task);

                        // TODO: error handling. show popup on failure to save?
                        let db_info: DatabaseInfo = (&state.database).into();
                        db_info.write(&state.path).unwrap();
                    }
                    true
                }
                _ => false,
            }
        } else if self.search_box_depend_on.is_open() {
            // popup is open
            match key.code {
                KeyCode::Enter => {
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
                }
                _ => false,
            }
        } else {
            match (key.code, key.modifiers) {
                (KeyCode::Char('c'), KeyModifiers::NONE) => {
                    self.task_popup.open();
                    true
                }
                (KeyCode::Char('d'), KeyModifiers::NONE) if !tasks.is_empty() => {
                    // delete
                    state.database.tasks.remove_node(tasks[self.index].0);

                    // TODO: error handling. show popup on failure to save?
                    let db_info: DatabaseInfo = (&state.database).into();
                    db_info.write(&state.path).unwrap();

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
