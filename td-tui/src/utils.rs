use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    layout::Rect,
    text::{Span, Spans},
};
use tui_input::InputRequest;

pub trait RectExt {
    /// Creates a new rect with the given width, starting at the same origin.
    fn take_x(self, amount: u16) -> Self;
    /// Creates a new rect with the given height, starting at the same origin.
    fn take_y(self, amount: u16) -> Self;
    fn skip_x(self, amount: u16) -> Self;
    fn skip_y(self, amount: u16) -> Self;
    fn take_last_x(self, amount: u16) -> Self;
    fn take_last_y(self, amount: u16) -> Self;
    fn skip_last_x(self, amount: u16) -> Self;
    fn skip_last_y(self, amount: u16) -> Self;

    /// Creates a rect in the center of this one.
    fn center_rect(&self, width: u16, height: u16) -> Self;
}

impl RectExt for Rect {
    fn take_x(self, amount: u16) -> Self {
        Self::new(self.x, self.y, amount, self.height)
    }

    fn take_y(self, amount: u16) -> Self {
        Self::new(self.x, self.y, self.width, amount)
    }

    fn skip_x(self, amount: u16) -> Self {
        Self::new(self.x + amount, self.y, self.width - amount, self.height)
    }

    fn skip_y(self, amount: u16) -> Self {
        Self::new(self.x, self.y + amount, self.width, self.height - amount)
    }
    fn take_last_x(self, amount: u16) -> Self {
        Self::new(self.x + self.width - amount, self.y, amount, self.height)
    }

    fn take_last_y(self, amount: u16) -> Self {
        Self::new(self.x, self.y + self.height - amount, self.width, amount)
    }

    fn skip_last_x(self, amount: u16) -> Self {
        Self::new(self.x, self.y, self.width - amount, self.height)
    }

    fn skip_last_y(self, amount: u16) -> Self {
        Self::new(self.x, self.y, self.width, self.height - amount)
    }

    fn center_rect(&self, width: u16, height: u16) -> Self {
        let center_x = self.x + self.width / 2;
        let center_y = self.y + self.height / 2;
        Self {
            x: center_x - width / 2,
            y: center_y - height / 2,
            width,
            height,
        }
    }
}

pub fn wrap_spans<'span>(
    spans: impl IntoIterator<Item = Span<'span>>,
    width: u16,
) -> Vec<Spans<'span>> {
    let mut current_width = 0;

    let mut ret = vec![Spans::default()];

    for span in spans {
        let span_len = span.content.len();

        if (current_width + span_len) as u16 > width {
            current_width = 0;
            ret.push(Spans::default());
        }

        current_width += span_len;
        ret.last_mut().unwrap().0.push(span);
    }

    ret
}

pub fn process_textbox_input(key: &KeyEvent) -> Option<InputRequest> {
    let ctrl_held = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Backspace if ctrl_held => Some(InputRequest::DeletePrevWord),
        KeyCode::Delete if ctrl_held => Some(InputRequest::DeleteNextWord),
        KeyCode::Backspace => Some(InputRequest::DeletePrevChar),
        KeyCode::Delete => Some(InputRequest::DeleteNextChar),

        KeyCode::Left if ctrl_held => Some(InputRequest::GoToPrevWord),
        KeyCode::Right if ctrl_held => Some(InputRequest::GoToNextWord),
        KeyCode::Left => Some(InputRequest::GoToPrevChar),
        KeyCode::Right => Some(InputRequest::GoToNextChar),
        KeyCode::Home => Some(InputRequest::GoToStart),
        KeyCode::End => Some(InputRequest::GoToEnd),

        KeyCode::Char(c) => Some(InputRequest::InsertChar(c)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const START_RECT: Rect = Rect {
        x: 100,
        y: 100,
        width: 10,
        height: 20,
    };

    #[test]
    fn test_take_x() {
        assert_eq!(START_RECT.take_x(3), Rect::new(100, 100, 3, 20));
    }

    #[test]
    fn test_take_y() {
        assert_eq!(START_RECT.take_y(3), Rect::new(100, 100, 10, 3));
    }

    #[test]
    fn test_skip_x() {
        assert_eq!(START_RECT.skip_x(3), Rect::new(103, 100, 7, 20));
    }

    #[test]
    fn test_skip_y() {
        assert_eq!(START_RECT.skip_y(3), Rect::new(100, 103, 10, 17));
    }

    #[test]
    fn test_take_last_x() {
        assert_eq!(START_RECT.take_last_x(3), Rect::new(107, 100, 3, 20));
    }

    #[test]
    fn test_take_last_y() {
        assert_eq!(START_RECT.take_last_y(3), Rect::new(100, 117, 10, 3));
    }

    #[test]
    fn test_skip_last_x() {
        assert_eq!(START_RECT.skip_last_x(3), Rect::new(100, 100, 7, 20));
    }

    #[test]
    fn test_skip_last_y() {
        assert_eq!(START_RECT.skip_last_y(3), Rect::new(100, 100, 10, 17));
    }

    #[test]
    fn test_center_rect() {
        assert_eq!(START_RECT.center_rect(6, 4), Rect::new(102, 108, 8, 12));
    }
}
