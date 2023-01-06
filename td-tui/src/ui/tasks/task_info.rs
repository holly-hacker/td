use td_lib::{
    database::Task,
    petgraph::{stable_graph::NodeIndex, visit::EdgeRef},
    time::{format_description, UtcOffset},
};
use tui::{
    text::{Span, Spans},
    widgets::Paragraph,
};

use crate::ui::{
    constants::{BOLD, BOLD_UNDERLINED},
    AppState, Component,
};

pub struct TaskInfo {
    task_index: Option<NodeIndex>,
}

impl TaskInfo {
    pub fn new(index: Option<NodeIndex>) -> Self {
        Self { task_index: index }
    }

    fn get_dependencies(node_index: NodeIndex, state: &AppState) -> Vec<&Task> {
        state
            .database
            .tasks
            .edges_directed(node_index, td_lib::petgraph::Direction::Outgoing)
            .map(|e| {
                state
                    .database
                    .tasks
                    .node_weight(e.target())
                    .expect("get node weight")
            })
            .collect()
    }

    fn get_dependents(node_index: NodeIndex, state: &AppState) -> Vec<&Task> {
        state
            .database
            .tasks
            .edges_directed(node_index, td_lib::petgraph::Direction::Incoming)
            .map(|e| {
                state
                    .database
                    .tasks
                    .node_weight(e.source())
                    .expect("get node weight")
            })
            .collect::<Vec<_>>()
    }
}

impl Component for TaskInfo {
    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        state: &AppState,
    ) {
        let Some(node_index) = self.task_index else {
            frame.render_widget(Paragraph::new("No task selected"), area);
            return;
        };

        let Some(task) = state.database.tasks.node_weight(node_index) else {
            frame.render_widget(Paragraph::new("Error: Task not found"), area);
            return;
        };

        let time_local = task
            .time_created
            .to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
        let date_format =
            format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
                .expect("valid hardcoded time format");

        // show useful info
        let mut spans = vec![
            Spans::from(Span::styled("Task info", BOLD_UNDERLINED)),
            Spans::from(vec![Span::styled("Name: ", BOLD), Span::raw(&task.title)]),
            Spans::from(vec![
                Span::styled("Created: ", BOLD),
                Span::raw(time_local.format(&date_format).unwrap()),
            ]),
        ];

        // add dependencies
        let dependencies = Self::get_dependencies(node_index, state);
        if !dependencies.is_empty() {
            spans.extend([
                Spans::default(),
                Spans::from(Span::styled("Depends on:", BOLD)),
            ]);

            spans.extend(
                dependencies
                    .iter()
                    .map(|d_val| Spans::from(vec![Span::raw("- "), Span::raw(&d_val.title)])),
            );
        }

        // add dependents
        let dependents = Self::get_dependents(node_index, state);
        if !dependents.is_empty() {
            spans.extend([
                Spans::default(),
                Spans::from(Span::styled("Depended on by:", BOLD)),
            ]);

            spans.extend(
                dependents
                    .iter()
                    .map(|d_val| Spans::from(vec![Span::raw("- "), Span::raw(&d_val.title)])),
            );
        }

        let paragraph = Paragraph::new(spans);
        frame.render_widget(paragraph, area);
    }

    fn update(&mut self, _key: crossterm::event::KeyEvent, _state: &mut AppState) -> bool {
        false
    }
}
