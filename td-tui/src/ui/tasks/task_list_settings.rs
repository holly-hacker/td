use crossterm::event::KeyCode;
use tui::widgets::Paragraph;

use crate::{
    ui::{
        constants::{LIST_HIGHLIGHT_STYLE, TEXT},
        Component,
    },
    utils::RectExt,
};

#[derive(Default)]
pub struct TaskListSettings {
    index: usize,
}

impl TaskListSettings {
    const CONTROL_COUNT: usize = 1;

    const INDEX_SORT_OLDEST: usize = 0;
}

impl Component for TaskListSettings {
    fn pre_render(
        &self,
        _global_state: &crate::ui::AppState,
        frame_storage: &mut crate::ui::FrameLocalStorage,
    ) {
        frame_storage.add_keybind("⇅", "Navigate list", Self::CONTROL_COUNT > 1);

        if self.index == Self::INDEX_SORT_OLDEST {
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
            Paragraph::new(format!(
                "[{}] Show oldest first",
                if state.sort_oldest_first { 'X' } else { ' ' }
            ))
            .style(if self.index == Self::INDEX_SORT_OLDEST {
                LIST_HIGHLIGHT_STYLE
            } else {
                TEXT
            }),
            area.take_y(1),
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
                self.index = self.index.saturating_sub(1).min(Self::CONTROL_COUNT - 1);
                return true;
            }
            KeyCode::Down => {
                self.index = self.index.saturating_add(1).min(Self::CONTROL_COUNT - 1);
                return true;
            }
            _ => (),
        };

        if self.index == Self::INDEX_SORT_OLDEST && key.code == KeyCode::Char(' ') {
            state.sort_oldest_first = !state.sort_oldest_first;
            return true;
        }

        false
    }
}
