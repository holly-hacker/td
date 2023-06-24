use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::Paragraph,
};

use super::Component;

pub struct DirtyIndicator;

impl Component for DirtyIndicator {
    fn render(
        &self,
        frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
        area: ratatui::layout::Rect,
        state: &super::AppState,
        _frame_storage: &super::FrameLocalStorage,
    ) {
        let text = if state.database.is_dirty() { "*" } else { " " };
        let p = Paragraph::new(Span::styled(
            text,
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        ));
        frame.render_widget(p, area);
    }
}
