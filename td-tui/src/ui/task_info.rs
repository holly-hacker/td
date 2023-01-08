use td_lib::{
    database::Task,
    petgraph::{stable_graph::NodeIndex, visit::EdgeRef},
    time::{format_description, UtcOffset},
};
use tui::{
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::ui::{
    constants::{BOLD, COMPLETED_TASK},
    AppState, Component, FrameLocalStorage,
};

pub struct TaskInfoDisplay;

impl TaskInfoDisplay {
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

impl Component for TaskInfoDisplay {
    fn pre_render(&self, _global_state: &AppState, _frame_storage: &mut FrameLocalStorage) {}

    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        state: &AppState,
        frame_storage: &FrameLocalStorage,
    ) {
        let block = Block::default()
            .title("Task Info")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let Some(node_index) = frame_storage.selected_task_index else {
            frame.render_widget(Paragraph::new("No task selected").block(block), area);
            return;
        };

        let Some(task) = state.database.tasks.node_weight(node_index) else {
            frame.render_widget(Paragraph::new("Error: Task not found").block(block), area);
            return;
        };

        let date_format =
            format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
                .expect("valid hardcoded time format");
        let time_local = task
            .time_created
            .to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));

        // show useful info
        let mut spans = vec![
            Spans::from(vec![Span::styled("Name: ", BOLD), Span::raw(&task.title)]),
            Spans::from(vec![
                Span::styled("Created: ", BOLD),
                Span::raw(time_local.format(&date_format).unwrap()),
            ]),
        ];

        if let Some(completed_at) = &task.time_completed {
            let time_local =
                completed_at.to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
            spans.push(Spans::from(vec![
                Span::styled("Completed at ", BOLD),
                Span::raw(time_local.format(&date_format).unwrap()),
            ]));
        }

        // add tags
        if !task.tags.is_empty() {
            spans.extend([Spans::default(), Spans::from(Span::styled("Tags:", BOLD))]);

            spans.extend(
                task.tags
                    .iter()
                    .map(|tag| Spans::from(vec![Span::raw("- "), Span::raw(tag)])),
            );
        }

        // add dependencies
        let dependencies = Self::get_dependencies(node_index, state);
        if !dependencies.is_empty() {
            spans.extend([
                Spans::default(),
                Spans::from(Span::styled("Depends on:", BOLD)),
            ]);

            spans.extend(dependencies.iter().map(|task| {
                Spans::from(vec![
                    Span::raw("- "),
                    if task.time_completed.is_some() {
                        Span::styled(&task.title, COMPLETED_TASK)
                    } else {
                        Span::raw(&task.title)
                    },
                ])
            }));
        }

        // add dependents
        let dependents = Self::get_dependents(node_index, state);
        if !dependents.is_empty() {
            spans.extend([
                Spans::default(),
                Spans::from(Span::styled("Depended on by:", BOLD)),
            ]);

            spans.extend(dependents.iter().map(|task| {
                Spans::from(vec![
                    Span::raw("- "),
                    if task.time_completed.is_some() {
                        Span::styled(&task.title, COMPLETED_TASK)
                    } else {
                        Span::raw(&task.title)
                    },
                ])
            }));
        }

        let paragraph = Paragraph::new(spans).block(block);
        frame.render_widget(paragraph, area);
    }

    fn process_input(
        &mut self,
        _key: crossterm::event::KeyEvent,
        _state: &mut AppState,
        _frame_storage: &FrameLocalStorage,
    ) -> bool {
        false
    }
}
