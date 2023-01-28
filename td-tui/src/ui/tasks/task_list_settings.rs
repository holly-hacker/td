use crossterm::event::KeyCode;
use tui::widgets::Paragraph;

use crate::{
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
    pub const UI_HEIGHT: u16 = 5;

    const SETTING_COUNT: usize = 2;

    const INDEX_SORT_OLDEST: usize = 0;
    const INDEX_FILTER_COMPLETED: usize = 1;
}

impl Component for TaskListSettings {
    fn pre_render(
        &self,
        _global_state: &crate::ui::AppState,
        frame_storage: &mut crate::ui::FrameLocalStorage,
    ) {
        frame_storage.add_keybind("⇅", "Navigate list", Self::SETTING_COUNT > 1);

        if self.index == Self::INDEX_SORT_OLDEST || self.index == Self::INDEX_FILTER_COMPLETED {
            frame_storage.add_keybind(" ", "Toggle", true);
        }
    }

    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        state: &crate::ui::AppState,
        _frame_storage: &crate::ui::FrameLocalStorage,
    ) {
        frame.render_widget(
            Paragraph::new("Sorting:").style(SETTINGS_HEADER),
            area.skip_y(0).take_y(1).take_x("Sorting:".len() as u16),
        );
        frame.render_widget(
            Paragraph::new(format!(
                " [{}] Show oldest first",
                if state.sort_oldest_first { 'X' } else { ' ' }
            ))
            .style(if self.index == Self::INDEX_SORT_OLDEST {
                LIST_HIGHLIGHT_STYLE
            } else {
                NO_STYLE
            }),
            area.skip_y(1).take_y(1),
        );

        frame.render_widget(
            Paragraph::new("Filter:").style(SETTINGS_HEADER),
            area.skip_y(3).take_y(1).take_x("Filter:".len() as u16),
        );
        frame.render_widget(
            Paragraph::new(format!(
                " [{}] Hide completed",
                if state.filter_completed { 'X' } else { ' ' }
            ))
            .style(if self.index == Self::INDEX_FILTER_COMPLETED {
                LIST_HIGHLIGHT_STYLE
            } else {
                NO_STYLE
            }),
            area.skip_y(4).take_y(1),
        );
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        state: &mut crate::ui::AppState,
        _frame_storage: &crate::ui::FrameLocalStorage,
    ) -> bool {
        match key.code {
            KeyCode::Up => {
                self.index = self.index.saturating_sub(1).min(Self::SETTING_COUNT - 1);
                return true;
            }
            KeyCode::Down => {
                self.index = self.index.saturating_add(1).min(Self::SETTING_COUNT - 1);
                return true;
            }
            _ => (),
        };

        if self.index == Self::INDEX_SORT_OLDEST && key.code == KeyCode::Char(' ') {
            state.sort_oldest_first = !state.sort_oldest_first;
            return true;
        }

        if self.index == Self::INDEX_FILTER_COMPLETED && key.code == KeyCode::Char(' ') {
            state.filter_completed = !state.filter_completed;
            return true;
        }

        false
    }
}
