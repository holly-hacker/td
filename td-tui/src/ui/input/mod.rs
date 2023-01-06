use crossterm::event::{KeyCode, KeyModifiers};
use tui::widgets::Paragraph;
use tui_input::{Input, InputRequest};

use super::Component;

pub struct TextBoxComponent {
    input: Input,
    focused: bool,
}

impl TextBoxComponent {
    pub fn new_focused() -> Self {
        Self {
            input: Input::default(),
            focused: true,
        }
    }

    pub fn text(&self) -> &str {
        self.input.value()
    }
}

impl Component for TextBoxComponent {
    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        _state: &super::AppState,
    ) {
        let paragraph = Paragraph::new(self.input.to_string());
        frame.render_widget(paragraph, area);

        // TODO: cursor seems to flash and move around. show this differently? maybe put the cursor
        // position in the paragraph as styling?
        if self.focused {
            frame.set_cursor(area.x + self.input.visual_cursor() as u16, area.y);
        }
    }

    fn update(&mut self, key: crossterm::event::KeyEvent, _state: &mut super::AppState) -> bool {
        if !self.focused {
            return false;
        }

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
                self.input.handle(request);
                true
            }
            None => false,
        }
    }
}
