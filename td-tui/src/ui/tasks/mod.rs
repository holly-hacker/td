use predicates::BoxPredicate;
use td_lib::database::Task;
use tui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders},
};

use self::{task_info::TaskInfoDisplay, task_list::TaskList};
use super::{constants::FG_WHITE, Component};

mod task_info;
mod task_list;

pub struct TaskPage {
    list: TaskList,
}
impl TaskPage {
    pub(crate) fn new(filter: BoxPredicate<Task>) -> Self {
        Self {
            list: TaskList::new(filter),
        }
    }
}

impl Component for TaskPage {
    fn pre_render(
        &self,
        global_state: &super::AppState,
        frame_storage: &mut super::FrameLocalStorage,
    ) {
        self.list.pre_render(global_state, frame_storage);
    }

    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        state: &super::AppState,
        frame_storage: &super::FrameLocalStorage,
    ) {
        let layout = Layout::default()
            .constraints([Constraint::Percentage(67), Constraint::Percentage(33)])
            .direction(Direction::Horizontal)
            .split(area);

        let list_area = layout[0];
        let info_area = layout[1];

        // render task list
        let list_block = Block::default()
            .title("Tasks")
            .borders(Borders::ALL)
            .border_style(FG_WHITE)
            .border_type(BorderType::Rounded);
        let inner_list_area = list_block.inner(list_area);
        frame.render_widget(list_block, list_area);
        self.list
            .render(frame, inner_list_area, state, frame_storage);

        // render info
        let info_block = Block::default()
            .title("Task Info")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner_info_area = info_block.inner(info_area);
        frame.render_widget(info_block, info_area);
        TaskInfoDisplay.render(frame, inner_info_area, state, frame_storage);
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        state: &mut super::AppState,
        frame_storage: &super::FrameLocalStorage,
    ) -> bool {
        if self.list.process_input(key, state, frame_storage) {
            return true;
        }

        false
    }
}
