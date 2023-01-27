use tui::{
    text::{Span, Spans},
    widgets::Paragraph,
};
use tui_input::Input;

use crate::{
    ui::{
        constants::{TEXTBOX_STYLE, TEXTBOX_STYLE_BG},
        AppState, Component, FrameLocalStorage,
    },
    utils::{process_textbox_input, wrap_text},
};

pub struct MultilineTextBoxComponent {
    input: Input,
    focused: bool,
    has_background: bool,
}

impl MultilineTextBoxComponent {
    #[must_use]
    pub fn new_focused() -> Self {
        Self {
            focused: true,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_background(mut self, enabled: bool) -> Self {
        self.has_background = enabled;
        self
    }

    #[must_use]
    pub fn with_text(mut self, text: String) -> Self {
        self.input = Input::from(text);
        self
    }

    #[must_use]
    pub fn text(&self) -> &str {
        self.input.value()
    }

    #[must_use]
    pub fn text_wrapped(&self, width: u16) -> Vec<String> {
        wrap_text(self.input.value(), width)
    }

    fn get_text_position(naive_cursor_pos: usize, text_wrapped: &[String]) -> (u16, u16) {
        let (mut cursor_x, mut cursor_y) = (naive_cursor_pos, 0);
        loop {
            let Some(line) = text_wrapped.get(cursor_y) else {
                break;
            };
            let line_len = line.len();
            if cursor_x <= line_len {
                break;
            }
            cursor_x -= line_len;
            cursor_y += 1;
        }
        (cursor_x as u16, cursor_y as u16)
    }
}

impl Default for MultilineTextBoxComponent {
    fn default() -> Self {
        Self {
            input: Default::default(),
            focused: true,
            has_background: true,
        }
    }
}

impl Component for MultilineTextBoxComponent {
    fn pre_render(&self, _global_state: &AppState, _frame_storage: &mut FrameLocalStorage) {}

    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        _state: &crate::ui::AppState,
        _frame_storage: &crate::ui::FrameLocalStorage,
    ) {
        let text_wrapped = self.text_wrapped(area.width);
        let wrapped = text_wrapped
            .iter()
            .map(|string| Spans::from(Span::from(string.as_str())))
            .collect::<Vec<_>>();
        let paragraph = Paragraph::new(wrapped).style(if self.has_background {
            TEXTBOX_STYLE_BG
        } else {
            TEXTBOX_STYLE
        });
        frame.render_widget(paragraph, area);

        if self.focused {
            let (cursor_x, cursor_y) = Self::get_text_position(self.input.cursor(), &text_wrapped);

            frame.set_cursor(area.x + cursor_x, area.y + cursor_y);
        }
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        _state: &mut crate::ui::AppState,
        _frame_storage: &crate::ui::FrameLocalStorage,
    ) -> bool {
        if !self.focused {
            return false;
        }

        // TODO: handle up/down
        // TODO: handle enter and ctrl+enter

        match process_textbox_input(&key) {
            Some(request) => {
                self.input.handle(request);
                true
            }
            None => false,
        }
    }
}
