use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use predicates::{reflection::PredicateReflection, Predicate};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
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

    fn split_x(self, index: u16) -> (Self, Self)
    where
        Self: Sized;
    fn split_y(self, index: u16) -> (Self, Self)
    where
        Self: Sized;
    fn split_last_x(self, index: u16) -> (Self, Self)
    where
        Self: Sized;
    fn split_last_y(self, index: u16) -> (Self, Self)
    where
        Self: Sized;

    fn slice_x(self, range: impl RangeBounds<u16>) -> Self;
    fn slice_y(self, range: impl RangeBounds<u16>) -> Self;

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

    fn split_x(self, index: u16) -> (Self, Self) {
        (self.take_x(index), self.skip_x(index))
    }

    fn split_y(self, index: u16) -> (Self, Self) {
        (self.take_y(index), self.skip_y(index))
    }

    fn split_last_x(self, index: u16) -> (Self, Self) {
        (self.skip_last_x(index), self.take_last_x(index))
    }

    fn split_last_y(self, index: u16) -> (Self, Self) {
        (self.skip_last_y(index), self.take_last_y(index))
    }

    fn slice_x(self, range: impl RangeBounds<u16>) -> Self {
        let start = match range.start_bound() {
            Bound::Included(&x) => x,
            Bound::Excluded(&x) => x + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&x) => x + 1,
            Bound::Excluded(&x) => x,
            Bound::Unbounded => self.width,
        };
        self.skip_x(start).take_x(end - start)
    }

    fn slice_y(self, range: impl RangeBounds<u16>) -> Self {
        let start = match range.start_bound() {
            Bound::Included(&y) => y,
            Bound::Excluded(&y) => y + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&y) => y + 1,
            Bound::Excluded(&y) => y,
            Bound::Unbounded => self.height,
        };
        self.skip_y(start).take_y(end - start)
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

pub fn wrap_text(text: &str, width: u16) -> Vec<String> {
    // see process at https://docs.rs/textwrap/latest/textwrap/core/index.html
    // we need to do this manually because we want to retain whitespace at the end of lines
    use textwrap::{core::break_words, wrap_algorithms::wrap_first_fit, WordSeparator};
    let words = WordSeparator::AsciiSpace.find_words(text);
    let words = break_words(words, width as usize);
    let lines = wrap_first_fit(&words, &[width as f64]);
    let strings = lines.into_iter().map(|words| {
        words
            .iter()
            .map(|word| format!("{}{}", word.word, word.whitespace))
            .collect::<String>()
    });
    strings.collect()
}

pub fn wrap_spans<'span>(
    spans: impl IntoIterator<Item = Span<'span>>,
    width: u16,
) -> Vec<Line<'span>> {
    let mut current_width = 0;

    let mut ret = vec![Line::default()];

    for span in spans {
        let span_len = span.content.len();

        if (current_width + span_len) as u16 > width {
            current_width = 0;
            ret.push(Line::default());
        }

        current_width += span_len;
        ret.last_mut().unwrap().spans.push(span);
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

/// A predicate to adapt another one by mapping its input.
///
/// See also https://github.com/assert-rs/predicates-rs/issues/142
pub struct MapPredicate<M, Item, Input, F>
where
    M: Predicate<Item>,
    F: Fn(&Input) -> &Item,
{
    inner: M,
    map_fn: F,
    _phantom_item: PhantomData<Item>,
    _phantom_input: PhantomData<Input>,
}

impl<M, Item, Input, F> MapPredicate<M, Item, Input, F>
where
    M: Predicate<Item>,
    F: Fn(&Input) -> &Item,
{
    pub fn new(inner: M, map_fn: F) -> Self {
        Self {
            inner,
            map_fn,
            _phantom_item: PhantomData,
            _phantom_input: PhantomData,
        }
    }
}

impl<M, Item, Input, F> Display for MapPredicate<M, Item, Input, F>
where
    M: Predicate<Item>,
    F: Fn(&Input) -> &Item,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MapPredicate")
    }
}

impl<M, Item, Input, F> PredicateReflection for MapPredicate<M, Item, Input, F>
where
    M: Predicate<Item>,
    F: Fn(&Input) -> &Item,
{
}

impl<M, Item, Input, F> Predicate<Input> for MapPredicate<M, Item, Input, F>
where
    M: Predicate<Item>,
    F: Fn(&Input) -> &Item,
{
    fn eval(&self, variable: &Input) -> bool {
        let mapped = (self.map_fn)(variable);
        self.inner.eval(mapped)
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
    fn test_split_x() {
        assert_eq!(
            START_RECT.split_x(2),
            (Rect::new(100, 100, 2, 20), Rect::new(102, 100, 8, 20))
        );
    }

    #[test]
    fn test_split_y() {
        assert_eq!(
            START_RECT.split_y(2),
            (Rect::new(100, 100, 10, 2), Rect::new(100, 102, 10, 18))
        );
    }

    #[test]
    fn test_split_last_x() {
        assert_eq!(
            START_RECT.split_last_x(2),
            (Rect::new(100, 100, 8, 20), Rect::new(108, 100, 2, 20))
        );
    }

    #[test]
    fn test_split_last_y() {
        assert_eq!(
            START_RECT.split_last_y(2),
            (Rect::new(100, 100, 10, 18), Rect::new(100, 118, 10, 2))
        );
    }

    #[test]
    fn test_slice_x() {
        assert_eq!(START_RECT.slice_x(..), START_RECT);
        assert_eq!(START_RECT.slice_x(0..), START_RECT);
        assert_eq!(START_RECT.slice_x(..10), START_RECT);
        assert_eq!(START_RECT.slice_x(0..10), START_RECT);
        assert_eq!(START_RECT.slice_x(..=9), START_RECT);
        assert_eq!(START_RECT.slice_x(0..=9), START_RECT);

        assert_eq!(START_RECT.slice_x(2..8), Rect::new(102, 100, 6, 20));
        assert_eq!(START_RECT.slice_x(2..=7), Rect::new(102, 100, 6, 20));
    }

    #[test]
    fn test_slice_y() {
        assert_eq!(START_RECT.slice_y(..), START_RECT);
        assert_eq!(START_RECT.slice_y(0..), START_RECT);
        assert_eq!(START_RECT.slice_y(..20), START_RECT);
        assert_eq!(START_RECT.slice_y(0..20), START_RECT);
        assert_eq!(START_RECT.slice_y(..=19), START_RECT);
        assert_eq!(START_RECT.slice_y(0..=19), START_RECT);

        assert_eq!(START_RECT.slice_y(2..18), Rect::new(100, 102, 10, 16));
        assert_eq!(START_RECT.slice_y(2..=17), Rect::new(100, 102, 10, 16));
    }

    #[test]
    fn test_center_rect() {
        assert_eq!(START_RECT.center_rect(6, 4), Rect::new(102, 108, 6, 4));
    }
}
