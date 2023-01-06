use crossterm::event::KeyCode;
use tui::{
    style::{Color, Style},
    symbols,
    text::Spans,
    widgets::Tabs,
};

use super::Component;
use crate::utils::RectExt;

pub struct TabLayout {
    items: Vec<Box<dyn Component>>,
    titles: Vec<&'static str>,
    index: usize,
}

impl TabLayout {
    pub fn new(items: impl IntoIterator<Item = (&'static str, Box<dyn Component>)>) -> Self {
        let (titles, items): (Vec<_>, Vec<_>) = items.into_iter().unzip();
        Self {
            titles,
            items,
            index: 0,
        }
    }
}

impl Component for TabLayout {
    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        state: &super::AppState,
    ) {
        let area_tabs = area.take_y(1);
        let area_content = area.skip_y(1);

        let titles = self
            .titles
            .iter()
            .enumerate()
            .map(|(i, v)| format!("{}: {v}", i + 1))
            .map(Spans::from)
            .collect();
        let tabs = Tabs::new(titles)
            .select(self.index)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(Style::default().fg(Color::White))
            .divider(symbols::DOT);

        frame.render_widget(tabs, area_tabs);

        if let Some(content) = self.items.get(self.index) {
            content.render(frame, area_content, state);
        }
    }

    fn update(&mut self, key: crossterm::event::KeyEvent, state: &mut super::AppState) -> bool {
        let content_update = if let Some(content) = self.items.get_mut(self.index) {
            content.update(key, state)
        } else {
            false
        };

        content_update
            || match key.code {
                KeyCode::Char(c @ '1'..='9') => {
                    let index = (c as u8 - b'1') as usize;
                    if index < self.items.len() {
                        self.index = index;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
    }
}
