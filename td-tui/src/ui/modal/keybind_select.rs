use std::borrow::Cow;

use tui::{
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    keybinds::*,
    ui::{constants::KEYBINDS_CHAR_ACTIVE, Component},
    utils::RectExt,
};

pub struct KeybindSelectModal {
    title: String,
    keybinds: Option<Vec<SimpleKeybind>>,
    selected_keybind: Option<SimpleKeybind>,
}

impl KeybindSelectModal {
    pub fn new(title: String) -> Self {
        Self {
            title,
            keybinds: None,
            selected_keybind: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.keybinds.is_some()
    }

    pub fn open(&mut self, keybinds: Vec<SimpleKeybind>) {
        self.keybinds = Some(keybinds);
        self.selected_keybind = None;
    }

    pub fn take_selected_keybind(&mut self) -> Option<SimpleKeybind> {
        let taken = self.selected_keybind.take();
        if taken.is_some() {
            self.close();
        }
        taken
    }

    pub fn close(&mut self) {
        self.keybinds = None;
    }
}

impl Component for KeybindSelectModal {
    fn pre_render(
        &self,
        _global_state: &crate::ui::AppState,
        frame_storage: &mut crate::ui::FrameLocalStorage,
    ) {
        if let Some(keybinds) = &self.keybinds {
            for keybind in keybinds {
                frame_storage.register_keybind(keybind, true);
            }
            frame_storage.register_keybind(KEYBIND_MODAL_CANCEL, true);
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
        let Some(keybinds) = &self.keybinds else {return;};

        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL);

        let spans = keybinds
            .iter()
            .map(|k| {
                Spans::from(vec![
                    Span::raw("["),
                    Span::styled(k.key_hint(), KEYBINDS_CHAR_ACTIVE),
                    Span::raw("] "),
                    Span::raw(k.description().cloned().unwrap_or(Cow::Borrowed(""))),
                ])
            })
            .collect::<Vec<_>>();

        let inner_width = (spans
            .iter()
            .map(|x| x.width())
            .max()
            .unwrap_or_default()
            .max(self.title.len())) as u16;
        let inner_height = spans.len() as u16;

        let paragraph = Paragraph::new(spans);

        let block_area = area.center_rect(inner_width + 2, inner_height + 2);
        let block_area_inner = block.inner(block_area);

        frame.render_widget(Clear, block_area);
        frame.render_widget(block, block_area);

        frame.render_widget(paragraph, block_area_inner);
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        _state: &mut crate::ui::AppState,
        _frame_storage: &crate::ui::FrameLocalStorage,
    ) -> bool {
        if self.is_open() && KEYBIND_MODAL_CANCEL.is_match(key) {
            self.close();
            return true;
        }

        if let Some(keybinds) = &self.keybinds {
            let found = keybinds.iter().find(|k| k.is_match(key));
            self.selected_keybind = found.cloned();
            // NOTE: still returning false so that the parent gets a chance to check if a keybind
            // is selected.
            return false;
        }

        false
    }
}
