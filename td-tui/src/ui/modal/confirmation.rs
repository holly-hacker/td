use crossterm::event::KeyCode;
use tui::{
    layout::Alignment,
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    ui::{
        constants::{MIN_MODAL_WIDTH, TEXT, TEXT_INVERTED},
        Component,
    },
    utils::{wrap_text, RectExt},
};

pub struct ConfirmationModal {
    title: Option<String>,
    text: String,
    selected_value: Option<bool>,
}

impl ConfirmationModal {
    pub fn new(text: String) -> Self {
        Self {
            text,
            title: None,
            selected_value: None,
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn is_open(&self) -> bool {
        self.selected_value.is_some()
    }

    pub fn open(&mut self, default_value: bool) {
        self.selected_value = Some(default_value);
    }

    pub fn close(&mut self) -> bool {
        self.selected_value.take().unwrap_or_default()
    }
}

impl Component for ConfirmationModal {
    fn pre_render(
        &self,
        _global_state: &crate::ui::AppState,
        frame_storage: &mut crate::ui::FrameLocalStorage,
    ) {
        if self.is_open() {
            frame_storage.add_keybind("⇆", "Choose option", true);
            frame_storage.add_keybind("⏎", "Select option", true);
            frame_storage.add_keybind("⎋", "Close", true);
            frame_storage.lock_keybinds();
        }
    }

    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        _state: &crate::ui::AppState,
        _frame_storage: &crate::ui::FrameLocalStorage,
    ) {
        let Some(selected_value) = self.selected_value else {return;};

        let mut block = Block::default().borders(Borders::ALL);
        if let Some(title) = &self.title {
            block = block.title(title.clone());
        }

        // create paragraph for yes/no selection
        const BUTTONS_LEN: usize = " <YES>  <NO>  ".len();
        let buttons = Paragraph::new(Spans::from(vec![
            Span::raw(" "),
            Span::styled("<YES>", if selected_value { TEXT_INVERTED } else { TEXT }),
            Span::raw("  "),
            Span::styled("<NO>", if !selected_value { TEXT_INVERTED } else { TEXT }),
            Span::raw("  "),
        ]))
        .alignment(Alignment::Center);

        let inner_width = MIN_MODAL_WIDTH
            .max(self.title.as_deref().unwrap_or_default().len() as u16)
            .max(BUTTONS_LEN as u16);
        let block_width = inner_width + 2;

        // wrap the text inside the inner width
        let wrapped_text = wrap_text(&self.text, inner_width)
            .into_iter()
            .map(|str| Spans::from(Span::from(str)))
            .collect::<Vec<_>>();
        let inner_height = wrapped_text.len() as u16 + 2;
        let block_height = inner_height + 2;

        // put the block in the center of the area
        let block_area = area.center_rect(block_width, block_height);
        let block_area_inner = block.inner(block_area);

        frame.render_widget(Clear, block_area);
        frame.render_widget(block, block_area);

        frame.render_widget(
            Paragraph::new(wrapped_text),
            block_area_inner.skip_last_y(1),
        );
        frame.render_widget(buttons, block_area_inner.take_last_y(1));
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        _state: &mut crate::ui::AppState,
        _frame_storage: &crate::ui::FrameLocalStorage,
    ) -> bool {
        if self.is_open() && key.code == KeyCode::Esc {
            self.close();
            return true;
        }

        let Some(selected_value) = &mut self.selected_value else {return false;};

        if let KeyCode::Left | KeyCode::Right = key.code {
            *selected_value = !*selected_value;
            return true;
        }

        false
    }
}
