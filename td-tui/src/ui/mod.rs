use std::{error::Error, io::Stdout, path::Path};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use td_lib::{
    database::{Database, DatabaseInfo},
    errors::DatabaseReadError,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

pub struct App {
    pub database: Database,
}
impl App {
    pub fn create(path: &Path) -> Result<Self, DatabaseReadError> {
        let db_info = if !path.exists() {
            println!("The given database file ({path:?}) does not exist, creating a new one.");

            let db_info = DatabaseInfo::default();
            db_info.write(path)?;
            db_info
        } else {
            DatabaseInfo::read(path)?
        };

        let database = db_info.try_into()?;

        Ok(Self { database })
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
            terminal.draw(|f| root_component.render(f, f.size(), &self.database))?;

            if let Event::Key(key) = event::read()? {
                let handled = root_component.update(key, &mut self.database);

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
    fn render(&self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, state: &Database);

    /// Update state based in a key event. Returns whether the key event is handled by this
    /// component or one of its children.
    fn update(&mut self, key: KeyEvent, state: &mut Database) -> bool;
}

struct BasicTaskList {
    index: usize,
    task_popup: BasicInputPopup,
}

impl Component for BasicTaskList {
    fn render(&self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, state: &Database) {
        // render the list
        let block = Block::default()
            .title("Basic Task List")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black));

        let list_items = state
            .items
            .iter()
            .map(|t| ListItem::new(t.as_str()))
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
        list_state.select(if state.items.is_empty() {
            None
        } else {
            Some(self.index)
        });
        f.render_stateful_widget(list, area, &mut list_state);

        // if needed, render the popup
        self.task_popup.render(f, area, state);
    }

    fn update(&mut self, key: KeyEvent, state: &mut Database) -> bool {
        self.index = self.index.clamp(0, state.items.len() - 1);

        if self.task_popup.update(key, state) {
            return true;
        }

        if self.task_popup.text.is_some() {
            // popup is open
            match key.code {
                KeyCode::Enter => {
                    if let Some(text) = self.task_popup.text.take() {
                        state.items.push(text);
                        // TODO: save state
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
                KeyCode::Up => {
                    if self.index != 0 {
                        self.index -= 1;
                    }
                    true
                }
                KeyCode::Down => {
                    if self.index != state.items.len() - 1 {
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
    fn render(&self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, _state: &Database) {
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

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(text.clone()).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn update(&mut self, key: KeyEvent, _state: &mut Database) -> bool {
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
