use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders, Clear},
    Frame,
};

use crate::{
    ui::{constants::MIN_MODAL_WIDTH, input::TextBoxComponent, AppState, Component},
    utils::RectExt,
};

pub struct TextInputModal {
    title: String,
    input: Option<TextBoxComponent>,
}

impl TextInputModal {
    pub fn new(title: String) -> TextInputModal {
        Self { title, input: None }
    }

    pub fn is_open(&self) -> bool {
        self.input.is_some()
    }

    pub fn open(&mut self) {
        self.input = Some(TextBoxComponent::new_focused());
    }

    pub fn close(&mut self) -> Option<String> {
        self.input.take().map(|input| input.text().to_string())
    }
}

impl Component for TextInputModal {
    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, state: &AppState) {
        let Some(textbox) = &self.input else {return;};

        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL);

        // put the block in the center of the area
        let block_width = MIN_MODAL_WIDTH
            .max(textbox.text().len() as u16 + 1)
            .max(self.title.len() as u16)
            + 2;
        let block_area = area.center_rect(block_width, TextBoxComponent::HEIGHT + 2);
        let block_area_inner = block.inner(block_area);

        frame.render_widget(Clear, block_area);
        frame.render_widget(block, block_area);
        textbox.render(frame, block_area_inner, state);
    }

    fn update(&mut self, key: KeyEvent, state: &mut AppState) -> bool {
        // always close with Esc
        if self.is_open() && key.code == KeyCode::Esc {
            self.close();
            return true;
        }

        let Some(input) = &mut self.input else {return false;};

        input.update(key, state)
    }
}
