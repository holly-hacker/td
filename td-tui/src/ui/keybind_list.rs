use tui::{
    symbols,
    text::{Span, Spans},
    widgets::{Paragraph, Wrap},
};

use super::{
    constants::{
        KEYBINDS_CHAR_ACTIVE, KEYBINDS_CHAR_INACTIVE, KEYBINDS_TEXT_ACTIVE, KEYBINDS_TEXT_INACTIVE,
    },
    Component,
};

pub struct KeybindList;

impl Component for KeybindList {
    fn pre_render(
        &self,
        _global_state: &super::AppState,
        _frame_storage: &mut super::FrameLocalStorage,
    ) {
    }

    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        _state: &super::AppState,
        frame_storage: &super::FrameLocalStorage,
    ) {
        let keybinds = &frame_storage.current_keybinds;

        let mut spans = vec![];

        let mut is_first = true;
        for (char, description, enabled) in keybinds {
            if !is_first {
                spans.push(Span::raw(" "));
                spans.push(Span::raw(symbols::DOT));
                spans.push(Span::raw(" "));
            }

            let style_text = if *enabled {
                KEYBINDS_TEXT_ACTIVE
            } else {
                KEYBINDS_TEXT_INACTIVE
            };
            let style_keybind = if *enabled {
                KEYBINDS_CHAR_ACTIVE
            } else {
                KEYBINDS_CHAR_INACTIVE
            };
            spans.push(Span::styled(description.to_string(), style_text));
            spans.push(Span::styled(" [", style_text));
            spans.push(Span::styled(char.to_string(), style_keybind));
            spans.push(Span::styled("]", style_text));

            is_first = false;
        }

        let paragraph = Paragraph::new(Spans::from(spans)).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    fn process_input(
        &mut self,
        _key: crossterm::event::KeyEvent,
        _state: &mut super::AppState,
        _frame_storage: &super::FrameLocalStorage,
    ) -> bool {
        false
    }
}
