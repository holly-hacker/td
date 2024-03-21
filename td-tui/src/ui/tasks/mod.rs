use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders},
};

use self::{task_info::TaskInfoDisplay, task_list::TaskList, task_list_settings::TaskListSettings};
use super::{
    constants::{FG_DIM, FG_LIGHT, FG_WHITE},
    Component,
};
use crate::{keybinds::*, utils::RectExt};

mod task_info;
mod task_list;
mod task_list_settings;
mod task_search;

pub struct TaskPage {
    list: TaskList,
    settings: TaskListSettings,
    selection_index: usize,
}

impl TaskPage {
    pub fn new() -> Self {
        Self {
            list: TaskList::new(),
            selection_index: 0,
            settings: TaskListSettings::default(),
        }
    }
}

impl Component for TaskPage {
    fn pre_render(
        &self,
        global_state: &super::AppState,
        frame_storage: &mut super::FrameLocalStorage,
    ) {
        if self.selection_index == 0 {
            self.list.pre_render(global_state, frame_storage);
            frame_storage.register_keybind(KEYBIND_TASKPAGE_PANE_SETTINGS, true);
        }
        if self.selection_index == 1 {
            self.settings.pre_render(global_state, frame_storage);
            frame_storage.register_keybind(KEYBIND_TASKPAGE_PANE_TASKS, true);
        }
    }

    fn render(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
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
            .style(if self.selection_index == 0 {
                FG_WHITE
            } else {
                FG_DIM
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner_list_area = list_block.inner(list_area);
        frame.render_widget(list_block, list_area);
        self.list
            .render(frame, inner_list_area, state, frame_storage);

        // split up the info area
        let (list_settings_area, task_info_area) =
            info_area.split_y(TaskListSettings::UI_HEIGHT + 2);

        // render list settings
        let list_settings_block = Block::default()
            .title("Task List Settings")
            .style(if self.selection_index == 1 {
                FG_WHITE
            } else {
                FG_DIM
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner_list_settings_area = list_settings_block.inner(list_settings_area);
        frame.render_widget(list_settings_block, list_settings_area);
        self.settings
            .render(frame, inner_list_settings_area, state, frame_storage);

        // render task info
        let task_info_block = Block::default()
            .title("Task Info")
            .style(FG_LIGHT)
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);
        let inner_task_info_area = task_info_block.inner(task_info_area);
        frame.render_widget(task_info_block, task_info_area);
        TaskInfoDisplay.render(frame, inner_task_info_area, state, frame_storage);
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        state: &mut super::AppState,
        frame_storage: &super::FrameLocalStorage,
    ) -> bool {
        if self.selection_index == 0 && self.list.process_input(key, state, frame_storage) {
            return true;
        }
        if self.selection_index == 1 && self.settings.process_input(key, state, frame_storage) {
            return true;
        }

        // if not handled by selected pane
        if KEYBIND_TASKPAGE_PANE_TASKS.is_match(key) {
            self.selection_index = 0;
            true
        } else if KEYBIND_TASKPAGE_PANE_SETTINGS.is_match(key) {
            self.selection_index = 1;
            true
        } else {
            false
        }
    }
}
