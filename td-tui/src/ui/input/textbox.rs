use tui::widgets::Paragraph;
use tui_input::Input;

use crate::{
    ui::{
        constants::{TEXTBOX_STYLE, TEXTBOX_STYLE_BG},
        AppState, Component, FrameLocalStorage,
    },
    utils::process_textbox_input,
};

/// A single-line text input field.
pub struct TextBoxComponent {
    input: Input,
    focused: bool,
    has_background: bool,
}

impl TextBoxComponent {
    pub const HEIGHT: u16 = 1;

    #[must_use]
    pub fn new_focused() -> Self {
        Self {
            focused: true,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_focus(mut self, enabled: bool) -> Self {
        self.focused = enabled;
        self
    }

    #[must_use]
    pub fn with_background(mut self, enabled: bool) -> Self {
        self.has_background = enabled;
        self
    }

    #[must_use]
    #[allow(unused)]
    pub fn with_text(mut self, text: String) -> Self {
        self.input = Input::from(text);
        self
    }

    #[must_use]
    pub fn text(&self) -> &str {
        self.input.value()
    }

    pub fn set_focus(&mut self, value: bool) {
        self.focused = value;
    }
}

impl Default for TextBoxComponent {
    fn default() -> Self {
        Self {
            input: Default::default(),
            focused: true,
            has_background: false,
        }
    }
}

impl Component for TextBoxComponent {
    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        _state: &AppState,
        _frame_storage: &FrameLocalStorage,
    ) {
        let paragraph = Paragraph::new(self.input.to_string()).style(if self.has_background {
            TEXTBOX_STYLE_BG
        } else {
            TEXTBOX_STYLE
        });
        frame.render_widget(paragraph, area);

        if self.focused {
            frame.set_cursor(area.x + self.input.visual_cursor() as u16, area.y);
        }
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        _state: &mut AppState,
        _frame_storage: &FrameLocalStorage,
    ) -> bool {
        if !self.focused {
            return false;
        }

        match process_textbox_input(&key) {
            Some(request) => {
                self.input.handle(request);
                true
            }
            None => false,
        }
    }
}
