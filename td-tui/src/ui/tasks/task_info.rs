use ratatui::{
    text::{Line, Span},
    widgets::Paragraph,
};
use td_lib::time::{format_description, UtcOffset};

use crate::ui::{
    constants::{BOLD, COMPLETED_TASK},
    AppState, Component, FrameLocalStorage,
};

pub struct TaskInfoDisplay;

impl Component for TaskInfoDisplay {
    fn render(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        state: &AppState,
        frame_storage: &FrameLocalStorage,
    ) {
        let Some(task_id) = frame_storage.selected_task_id.clone() else {
            frame.render_widget(Paragraph::new("No task selected"), area);
            return;
        };

        let task = &state.database[&task_id];

        let date_format =
            format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
                .expect("valid hardcoded time format");
        let time_local = task
            .time_created
            .to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));

        // show useful info
        let mut spans = vec![
            Line::from(vec![Span::styled("Name: ", BOLD), Span::raw(&task.title)]),
            Line::from(vec![
                Span::styled("Created: ", BOLD),
                Span::raw(time_local.format(&date_format).unwrap()),
            ]),
        ];

        if let Some(started_at) = &task.time_started {
            let time_local =
                started_at.to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
            spans.push(Line::from(vec![
                Span::styled("Started: ", BOLD),
                Span::raw(time_local.format(&date_format).unwrap()),
            ]));
        }

        if let Some(completed_at) = &task.time_completed {
            let time_local =
                completed_at.to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
            spans.push(Line::from(vec![
                Span::styled("Completed: ", BOLD),
                Span::raw(time_local.format(&date_format).unwrap()),
            ]));
        }

        // add tags
        if !task.tags.is_empty() {
            spans.extend([Line::default(), Line::from(Span::styled("Tags:", BOLD))]);

            spans.extend(
                task.tags
                    .iter()
                    .map(|tag| Line::from(vec![Span::raw("- "), Span::raw(tag)])),
            );
        }

        // add dependencies
        let mut dependencies = state.database.get_dependencies(&task_id).peekable();
        if dependencies.peek().is_some() {
            spans.extend([
                Line::default(),
                Line::from(Span::styled("Depends on:", BOLD)),
            ]);

            spans.extend(dependencies.map(|task| {
                Line::from(vec![
                    Span::raw("- "),
                    if task.time_completed.is_some() {
                        Span::styled(&task.title, COMPLETED_TASK)
                    } else {
                        Span::raw(&task.title)
                    },
                ])
            }));
        }

        // add inverse dependencies
        let mut dependents = state.database.get_inverse_dependencies(&task_id).peekable();
        if dependents.peek().is_some() {
            spans.extend([
                Line::default(),
                Line::from(Span::styled("Depended on by:", BOLD)),
            ]);

            spans.extend(dependents.map(|task| {
                Line::from(vec![
                    Span::raw("- "),
                    if task.time_completed.is_some() {
                        Span::styled(&task.title, COMPLETED_TASK)
                    } else {
                        Span::raw(&task.title)
                    },
                ])
            }));
        }

        frame.render_widget(Paragraph::new(spans), area);
    }
}
