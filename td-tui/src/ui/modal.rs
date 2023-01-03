use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use tui_input::{Input, InputRequest};

use super::{AppState, Component};

const MIN_WIDTH: usize = 16;

pub struct BasicInputPopup {
    title: String,
    input: Option<Input>,
}

impl BasicInputPopup {
    pub fn new(arg: String) -> BasicInputPopup {
        Self {
            title: arg,
            input: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.input.is_some()
    }

    pub fn open(&mut self) {
        self.input = Some(Input::default());
    }

    pub fn close(&mut self) -> Option<String> {
        self.input.take().map(|input| input.into())
    }
}

impl Component for BasicInputPopup {
    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, _state: &AppState) {
        let Some(text) = &self.input else {return;};

        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(text.to_string()).block(block);

        // put the block in the center of the area
        let block_width = MIN_WIDTH.max(text.value().len() + 1).max(self.title.len()) + 2;
        let area_center = (area.x + area.width / 2, area.y + area.height / 2);
        let block_area = Rect::new(
            area_center.0 - block_width as u16 / 2,
            area_center.1 - 1,
            block_width as u16,
            3,
        );

        frame.render_widget(Clear, block_area);
        frame.render_widget(paragraph, block_area);

        // TODO: cursor seems to flash and move around. show this differently? maybe put the cursor
        // position in the paragraph as styling?
        frame.set_cursor(
            block_area.x + 1 + text.visual_cursor() as u16,
            block_area.y + 1,
        );
    }

    fn update(&mut self, key: KeyEvent, _state: &mut AppState) -> bool {
        // always close with Esc
        if self.is_open() && key.code == KeyCode::Esc {
            self.close();
            return true;
        }

        let Some(input) = &mut self.input else {return false;};

        let ctrl_held = key.modifiers.contains(KeyModifiers::CONTROL);
        let request = match key.code {
            KeyCode::Backspace if ctrl_held => Some(InputRequest::DeletePrevWord),
            KeyCode::Delete if ctrl_held => Some(InputRequest::DeleteNextWord),
            KeyCode::Backspace => Some(InputRequest::DeletePrevChar),
            KeyCode::Delete => Some(InputRequest::DeleteNextChar),

            KeyCode::Left if ctrl_held => Some(InputRequest::GoToPrevWord),
            KeyCode::Right if ctrl_held => Some(InputRequest::GoToNextWord),
            KeyCode::Left => Some(InputRequest::GoToPrevChar),
            KeyCode::Right => Some(InputRequest::GoToNextChar),
            KeyCode::Up | KeyCode::Home => Some(InputRequest::GoToStart),
            KeyCode::Down | KeyCode::End => Some(InputRequest::GoToEnd),

            KeyCode::Char(c) => Some(InputRequest::InsertChar(c)),
            _ => None,
        };

        match request {
            Some(request) => {
                input.handle(request);
                true
            }
            None => false,
        }
    }
}
