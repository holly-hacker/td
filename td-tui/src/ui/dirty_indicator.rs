use tui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::Paragraph,
};

use super::Component;

pub struct DirtyIndicator;

impl Component for DirtyIndicator {
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
        state: &super::AppState,
        _frame_storage: &super::FrameLocalStorage,
    ) {
        let text = if state.is_dirty { "*" } else { " " };
        let p = Paragraph::new(Span::styled(
            text,
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        ));
        frame.render_widget(p, area);
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
