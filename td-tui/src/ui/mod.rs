use std::{error::Error, io::Stdout};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use td_lib::database::Database;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

pub struct App {
    pub database: Database,
}
impl App {
    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut root_component = BasicTaskList {
            tasks: self.database.items.clone(),
            index: 0,
        };

        loop {
            terminal.draw(|f| root_component.render(f, f.size()))?;

            if let Event::Key(key) = event::read()? {
                let handled = root_component.update(key);

                if !handled {
                    match key.code {
                        KeyCode::Char('q') => break,
                        _ => continue,
                    }
                }
            }
        }

        Ok(())
    }
}

trait Component {
    /// Render the component and its children to the given area.
    fn render(&self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect);

    /// Update state based in a key event. Returns whether the key event is handled by this
    /// component or one of its children.
    fn update(&mut self, key: KeyEvent) -> bool;
}

struct BasicTaskList {
    tasks: Vec<String>,
    index: usize,
}

impl Component for BasicTaskList {
    fn render(&self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let block = Block::default()
            .title("Basic Task List")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black));

        let list_items = self
            .tasks
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
        let mut state = ListState::default();
        state.select(Some(self.index));
        f.render_stateful_widget(list, area, &mut state);
    }

    fn update(&mut self, key: KeyEvent) -> bool {
        // do nothing for now
        match key.code {
            KeyCode::Up => {
                if self.index != 0 {
                    self.index -= 1;
                }
                true
            }
            KeyCode::Down => {
                if self.index != self.tasks.len() - 1 {
                    self.index += 1;
                }
                true
            }
            _ => false,
        }
    }
}
