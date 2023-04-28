use tui::widgets::Paragraph;

use crate::{
    keybinds::*,
    ui::{
        constants::{LIST_HIGHLIGHT_STYLE, NO_STYLE, SETTINGS_HEADER},
        Component,
    },
    utils::RectExt,
};

#[derive(Default)]
pub struct TaskListSettings {
    index: usize,
}

impl TaskListSettings {
    pub const UI_HEIGHT: u16 = Self::SETTING_COUNT as u16 + 2 + 1;

    const SETTING_COUNT: usize = 4;

    const INDEX_SORT_OLDEST: usize = 0;
    const INDEX_FILTER_COMPLETED: usize = 1;
    const INDEX_FILTER_UNACTIONABLE: usize = 2;
    const INDEX_FILTER_SEARCH: usize = 3;
}

impl Component for TaskListSettings {
    fn pre_render(
        &self,
        _global_state: &crate::ui::AppState,
        frame_storage: &mut crate::ui::FrameLocalStorage,
    ) {
        frame_storage.register_keybind(KEYBIND_CONTROLS_LIST_NAV, Self::SETTING_COUNT > 1);

        if self.index == Self::INDEX_SORT_OLDEST || self.index == Self::INDEX_FILTER_COMPLETED {
            frame_storage.register_keybind(KEYBIND_CONTROLS_CHECKBOX_TOGGLE, true);
        }
    }

    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        state: &crate::ui::AppState,
        _frame_storage: &crate::ui::FrameLocalStorage,
    ) {
        let (area_sorting, area_filter) = area.split_y(3);

        let checkbox = |b: bool| if b { 'x' } else { ' ' };
        let list_style = |i: usize| {
            if self.index == i {
                LIST_HIGHLIGHT_STYLE
            } else {
                NO_STYLE
            }
        };

        // Sorting
        frame.render_widget(
            Paragraph::new("Sorting:").style(SETTINGS_HEADER),
            area_sorting.slice_y(0..=0).take_x("Sorting:".len() as u16),
        );
        frame.render_widget(
            Paragraph::new(format!(
                " [{}] Show oldest first",
                checkbox(state.sort_oldest_first)
            ))
            .style(list_style(Self::INDEX_SORT_OLDEST)),
            area_sorting.slice_y(1..=1),
        );

        // Filter
        frame.render_widget(
            Paragraph::new("Filter:").style(SETTINGS_HEADER),
            area_filter.slice_y(0..=0).take_x("Filter:".len() as u16),
        );
        frame.render_widget(
            Paragraph::new(format!(
                " [{}] Hide completed",
                checkbox(state.filter_completed)
            ))
            .style(list_style(Self::INDEX_FILTER_COMPLETED)),
            area_filter.slice_y(1..=1),
        );
        frame.render_widget(
            Paragraph::new(format!(
                " [{}] Hide unactionable (unfinished dependencies)",
                checkbox(state.filter_unactionable)
            ))
            .style(list_style(Self::INDEX_FILTER_UNACTIONABLE)),
            area_filter.slice_y(2..=2),
        );
        frame.render_widget(
            Paragraph::new(format!(" [{}] Text search", checkbox(state.filter_search)))
                .style(list_style(Self::INDEX_FILTER_SEARCH)),
            area_filter.slice_y(3..=3),
        );
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        state: &mut crate::ui::AppState,
        _frame_storage: &crate::ui::FrameLocalStorage,
    ) -> bool {
        if let Some(key) = KEYBIND_CONTROLS_LIST_NAV.get_match(key) {
            match key {
                UpDownKey::Up => {
                    self.index = self.index.saturating_sub(1).min(Self::SETTING_COUNT - 1);
                    true
                }
                UpDownKey::Down => {
                    self.index = self.index.saturating_add(1).min(Self::SETTING_COUNT - 1);
                    true
                }
            }
        } else {
            match self.index {
                Self::INDEX_SORT_OLDEST if KEYBIND_CONTROLS_CHECKBOX_TOGGLE.is_match(key) => {
                    state.sort_oldest_first = !state.sort_oldest_first;
                    true
                }
                Self::INDEX_FILTER_COMPLETED if KEYBIND_CONTROLS_CHECKBOX_TOGGLE.is_match(key) => {
                    state.filter_completed = !state.filter_completed;
                    true
                }
                Self::INDEX_FILTER_UNACTIONABLE
                    if KEYBIND_CONTROLS_CHECKBOX_TOGGLE.is_match(key) =>
                {
                    state.filter_unactionable = !state.filter_unactionable;
                    true
                }
                Self::INDEX_FILTER_SEARCH if KEYBIND_CONTROLS_CHECKBOX_TOGGLE.is_match(key) => {
                    state.filter_search = !state.filter_search;
                    true
                }
                _ => false,
            }
        }
    }
}
