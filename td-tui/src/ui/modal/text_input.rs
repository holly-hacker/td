use std::io::Stdout;

use crossterm::event::KeyEvent;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders, Clear},
    Frame,
};

use crate::{
    keybinds::*,
    ui::{constants::MIN_MODAL_WIDTH, input::MultilineTextBoxComponent, AppState, Component},
    utils::RectExt,
};

pub struct TextInputModal {
    title: String,
    input: Option<MultilineTextBoxComponent>,
}

impl TextInputModal {
    pub fn new(title: String) -> Self {
        Self { title, input: None }
    }

    pub fn is_open(&self) -> bool {
        self.input.is_some()
    }

    pub fn open(&mut self) {
        self.input = Some(MultilineTextBoxComponent::new_focused().with_background(false));
    }

    pub fn open_with_text(&mut self, input: String) {
        self.input = Some(
            MultilineTextBoxComponent::new_focused()
                .with_background(false)
                .with_text(input),
        );
    }
    pub fn close(&mut self) -> Option<String> {
        self.input.take().map(|input| input.text().to_string())
    }
}

impl Component for TextInputModal {
    fn pre_render(
        &self,
        global_state: &AppState,
        frame_storage: &mut crate::ui::FrameLocalStorage,
    ) {
        if let Some(input) = &self.input {
            debug_assert!(self.is_open());
            input.pre_render(global_state, frame_storage);

            // NOTE: could check if at least 1 character is entered, as an option
            frame_storage.register_keybind(KEYBIND_MODAL_SUBMIT, true);
            frame_storage.register_keybind(KEYBIND_MODAL_CANCEL, true);
            frame_storage.lock_keybinds();
        }
    }

    fn render(
        &self,
        frame: &mut Frame<CrosstermBackend<Stdout>>,
        area: Rect,
        state: &AppState,
        frame_storage: &crate::ui::FrameLocalStorage,
    ) {
        let Some(textbox) = &self.input else {return;};

        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL);

        // put the block in the center of the area
        let block_width = MIN_MODAL_WIDTH.clamp(self.title.len() as u16, 32);
        let block_area = area.center_rect(
            block_width + 2,
            textbox.text_wrapped(block_width).len() as u16 + 2,
        );
        let block_area_inner = block.inner(block_area);

        frame.render_widget(Clear, block_area);
        frame.render_widget(block, block_area);
        textbox.render(frame, block_area_inner, state, frame_storage);
    }

    fn process_input(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
        frame_storage: &crate::ui::FrameLocalStorage,
    ) -> bool {
        // always close with Esc
        if self.is_open() && KEYBIND_MODAL_CANCEL.is_match(key) {
            self.close();
            return true;
        }

        let Some(input) = &mut self.input else {return false;};

        input.process_input(key, state, frame_storage)
    }
}
