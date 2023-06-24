use td_lib::database::Task;

use crate::ui::{input::TextBoxComponent, Component};

pub struct TaskSearchBarComponent {
    textbox: TextBoxComponent,
}

impl Default for TaskSearchBarComponent {
    fn default() -> Self {
        Self {
            textbox: TextBoxComponent::default()
                .with_background(true)
                .with_focus(false),
        }
    }
}

impl TaskSearchBarComponent {
    pub fn filter(&self, task: &Task) -> bool {
        // PERF: allocates new string every time which is fairly wasteful
        task.title
            .to_lowercase()
            .contains(&self.textbox.text().to_lowercase())
    }

    pub fn set_focus(&mut self, value: bool) {
        self.textbox.set_focus(value);
    }
}

impl Component for TaskSearchBarComponent {
    fn pre_render(
        &self,
        global_state: &crate::ui::AppState,
        frame_storage: &mut crate::ui::FrameLocalStorage,
    ) {
        self.textbox.pre_render(global_state, frame_storage);
    }

    fn render(
        &self,
        frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
        area: ratatui::layout::Rect,
        state: &crate::ui::AppState,
        frame_storage: &crate::ui::FrameLocalStorage,
    ) {
        self.textbox.render(frame, area, state, frame_storage);
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        state: &mut crate::ui::AppState,
        frame_storage: &crate::ui::FrameLocalStorage,
    ) -> bool {
        self.textbox.process_input(key, state, frame_storage)
    }
}
