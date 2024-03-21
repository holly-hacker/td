use ratatui::{symbols, text::Span, widgets::Paragraph};

use super::{
    constants::{
        KEYBINDS_CHAR_ACTIVE, KEYBINDS_CHAR_INACTIVE, KEYBINDS_TEXT_ACTIVE, KEYBINDS_TEXT_INACTIVE,
    },
    Component,
};
use crate::utils::wrap_spans;

pub struct KeybindList;

impl KeybindList {
    pub fn get_spans(frame_storage: &super::FrameLocalStorage) -> Vec<Span> {
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

        spans
    }
}

impl Component for KeybindList {
    fn render(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        _state: &super::AppState,
        frame_storage: &super::FrameLocalStorage,
    ) {
        let spans = wrap_spans(Self::get_spans(frame_storage), area.width);
        let paragraph = Paragraph::new(spans);
        frame.render_widget(paragraph, area);
    }
}
