use std::{error::Error, io::Stdout, path::PathBuf, time::SystemTime};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use td_lib::{
    database::{Database, DatabaseInfo, Task},
    errors::DatabaseReadError,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

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
        let mut root_component = BasicTaskList {
            index: 0,
            task_popup: BasicInputPopup {
                text: None,
                title: "Enter new task".into(),
            },
        };

        loop {
            terminal.draw(|f| root_component.render(f, f.size(), self))?;

            if let Event::Key(key) = event::read()? {
                let handled = root_component.update(key, self);
                if !handled {
                    match key.code {
                        KeyCode::Char('q') => break,
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

trait Component {
    /// Render the component and its children to the given area.
    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, state: &AppState);

    /// Update state based in a key event. Returns whether the key event is handled by this
    /// component or one of its children.
    fn update(&mut self, key: KeyEvent, state: &mut AppState) -> bool;
}

struct BasicTaskList {
    index: usize,
    task_popup: BasicInputPopup,
}

impl Component for BasicTaskList {
    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, state: &AppState) {
        let mut tasks = state.database.tasks.node_weights().collect::<Vec<_>>();
        tasks.sort_by(|a, b| a.time_created.cmp(&b.time_created));

        // render the list
        let block = Block::default()
            .title("Basic Task List")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black));

        let list_items = tasks
            .iter()
            .map(|t| ListItem::new(t.title.clone()))
            .collect::<Vec<_>>();
        let list = List::new(list_items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().fg(Color::DarkGray));
        let mut list_state = ListState::default();
        list_state.select(if tasks.is_empty() {
            None
        } else {
            Some(self.index)
        });
        frame.render_stateful_widget(list, area, &mut list_state);

        // if needed, render the popup
        self.task_popup.render(frame, area, state);
    }

    fn update(&mut self, key: KeyEvent, state: &mut AppState) -> bool {
        if self.task_popup.update(key, state) {
            return true;
        }

        let task_indices = state.database.tasks.node_indices().collect::<Vec<_>>();

        if !task_indices.is_empty() {
            self.index = self.index.clamp(0, task_indices.len() - 1);
        }

        if self.task_popup.text.is_some() {
            // popup is open
            match key.code {
                KeyCode::Enter => {
                    if let Some(text) = self.task_popup.text.take() {
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
                KeyCode::Esc => {
                    self.task_popup.text = None;
                    true
                }
                _ => false,
            }
        } else {
            match key.code {
                KeyCode::Char('c') => {
                    self.task_popup.text = Some(String::new());
                    true
                }
                KeyCode::Char('d') if !task_indices.is_empty() => {
                    state.database.tasks.remove_node(task_indices[self.index]);

                    // TODO: error handling. show popup on failure to save?
                    let db_info: DatabaseInfo = (&state.database).into();
                    db_info.write(&state.path).unwrap();

                    true
                }
                KeyCode::Up => {
                    if self.index != 0 {
                        self.index -= 1;
                    }
                    true
                }
                KeyCode::Down => {
                    if self.index != task_indices.len() - 1 {
                        self.index += 1;
                    }
                    true
                }
                _ => false,
            }
        }
    }
}

struct BasicInputPopup {
    title: String,
    text: Option<String>,
}

impl Component for BasicInputPopup {
    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, _state: &AppState) {
        let Some(text) = &self.text else {return;};

        let popup_area_vertical = Layout::default()
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .direction(Direction::Horizontal)
            .split(area)[1];
        let popup_area = Rect::new(
            popup_area_vertical.x,
            popup_area_vertical.height / 2 - 2,
            popup_area_vertical.width,
            3,
        );

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(text.clone()).block(block);
        frame.render_widget(paragraph, popup_area);
    }

    fn update(&mut self, key: KeyEvent, _state: &mut AppState) -> bool {
        let Some(text) = &mut self.text else {return false;};

        // TODO: use tui-input
        match key.code {
            KeyCode::Char(c) => {
                text.push(c);
                true
            }
            _ => false,
        }
    }
}
