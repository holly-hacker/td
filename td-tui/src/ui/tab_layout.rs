use crossterm::event::KeyCode;
use tui::{symbols, text::Spans, widgets::Tabs};

use super::{
    constants::{TAB_HIGHLIGHT_STYLE, TAB_STYLE},
    Component,
};
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

    fn get_selected_component(&self) -> Option<&dyn Component> {
        self.items.get(self.index).map(|x| x.as_ref())
    }

    fn get_selected_component_mut(&mut self) -> Option<&mut Box<dyn Component>> {
        self.items.get_mut(self.index)
    }
}

impl Component for TabLayout {
    fn pre_render(
        &self,
        global_state: &super::AppState,
        frame_storage: &mut super::FrameLocalStorage,
    ) {
        if let Some(component) = self.get_selected_component() {
            component.pre_render(global_state, frame_storage)
        }
    }

    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        state: &super::AppState,
        frame_storage: &super::FrameLocalStorage,
    ) {
        let area_tabs = area.take_y(1);
        let area_content = area.skip_y(1);

        let titles = self
            .titles
            .iter()
            .enumerate()
            .map(|(i, v)| format!("{v} [{}]", i + 1))
            .map(Spans::from)
            .collect();
        let tabs = Tabs::new(titles)
            .select(self.index)
            .style(TAB_STYLE)
            .highlight_style(TAB_HIGHLIGHT_STYLE)
            .divider(symbols::DOT);

        frame.render_widget(tabs, area_tabs);

        if let Some(content) = self.get_selected_component() {
            content.render(frame, area_content, state, frame_storage);
        }
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        state: &mut super::AppState,
        frame_storage: &super::FrameLocalStorage,
    ) -> bool {
        let content_update = if let Some(content) = self.get_selected_component_mut() {
            content.process_input(key, state, frame_storage)
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
