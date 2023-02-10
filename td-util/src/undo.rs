//! Provides generic undo functionality on an arbitrary state object.

use std::ops::Deref;

// TODO: trim start of stack to ensure memory usage doesn't grow out of control

/// A wrapper for a state, allowing rolling back changes using an undo-redo system.
///
/// This operates by keeping around copies of the state, with a pointer to the current state.
pub struct UndoWrapper<T: Clone> {
    states: Vec<T>,
    current_index: usize,
    clean_index: Option<usize>,
}

impl<T: Clone> UndoWrapper<T> {
    /// Create a new instance with the given state as the current (and only) state.
    pub fn new(initial_state: T) -> Self {
        Self {
            states: vec![initial_state],
            current_index: 0,
            clean_index: None,
        }
    }

    /// Gets a reference to the current state.
    #[must_use]
    pub fn state(&self) -> &T {
        debug_assert!(!self.states.is_empty());
        debug_assert!(self.states.len() > self.current_index);
        &self.states[self.current_index]
    }

    fn state_mut(&mut self) -> &mut T {
        debug_assert!(!self.states.is_empty());
        debug_assert!(self.states.len() > self.current_index);
        &mut self.states[self.current_index]
    }

    /// Gets a mutable reference to the current state. Doing this will create a new copy of the
    /// state that gets mutated, allowing calling undo to roll back to the previous state later.
    pub fn modify<F: FnOnce(&mut T)>(&mut self, func: F) {
        self.clear_redo_states();

        self.states.push(self.state().clone());
        self.current_index += 1;
        func(self.state_mut());
    }

    fn clear_redo_states(&mut self) {
        self.states.truncate(self.current_index + 1);

        if let Some(clean_index) = self.clean_index {
            if clean_index > self.current_index {
                self.clean_index = None;
            }
        }
    }

    /// Sets the current state pointer back one state, if possible. Returns `true` if the current
    /// state has changed.
    pub fn undo(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            true
        } else {
            false
        }
    }

    /// Returns how many times the state can be reverted.
    #[must_use]
    pub fn undo_count(&self) -> usize {
        self.current_index
    }

    /// Forwards the state one stage after calling [`Self::undo`]. This will only work right before
    /// an undo, modifying the current state using [`Self::modify`] will clear the redo queue.
    pub fn redo(&mut self) -> bool {
        if self.current_index < self.states.len() - 1 {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    /// Returns how many times the state can be forwarded.
    #[must_use]
    pub fn redo_count(&self) -> usize {
        self.states.len() - 1 - self.current_index
    }

    /// Marks the current state as the "clean" state. This can be used to keep track of which state
    /// is consistent with an externally saved one, such as the version "on disk".
    pub fn mark_clean(&mut self) {
        self.clean_index = Some(self.current_index);
    }

    /// Returns whether the current state is "dirty". See [`Self::mark_clean`].
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.clean_index != Some(self.current_index)
    }
}

impl<T: Clone + Default> Default for UndoWrapper<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> Deref for UndoWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo() {
        let mut undo = UndoWrapper::new(0i32);
        assert_eq!(undo.state(), &0);

        undo.modify(|x| *x += 1);
        assert_eq!(undo.state(), &1);

        undo.undo();
        assert_eq!(undo.state(), &0);
    }

    #[test]
    fn redo() {
        let mut undo = UndoWrapper::new(0i32);
        assert_eq!(undo.state(), &0);

        undo.modify(|x| *x += 1);
        assert_eq!(undo.state(), &1);

        undo.undo();
        assert_eq!(undo.state(), &0);

        undo.redo();
        assert_eq!(undo.state(), &1);
    }

    #[test]
    fn redo_multiple() {
        let mut undo = UndoWrapper::new(0i32);
        assert_eq!(undo.state(), &0);

        undo.modify(|x| *x += 1);
        assert_eq!(undo.state(), &1);

        undo.modify(|x| *x += 1);
        assert_eq!(undo.state(), &2);

        undo.undo();
        assert_eq!(undo.state(), &1);

        undo.undo();
        assert_eq!(undo.state(), &0);

        undo.redo();
        assert_eq!(undo.state(), &1);

        undo.redo();
        assert_eq!(undo.state(), &2);
    }

    #[test]
    fn undo_redo_undo_redo() {
        let mut undo = UndoWrapper::new(0i32);
        assert_eq!(undo.state(), &0);

        undo.modify(|x| *x += 1);
        assert_eq!(undo.state(), &1);

        undo.undo();
        assert_eq!(undo.state(), &0);

        undo.redo();
        assert_eq!(undo.state(), &1);

        undo.undo();
        assert_eq!(undo.state(), &0);

        undo.redo();
        assert_eq!(undo.state(), &1);
    }

    #[test]
    fn undo_count() {
        let mut undo = UndoWrapper::new(());

        assert_eq!(undo.undo_count(), 0);

        undo.modify(|_| ());
        assert_eq!(undo.undo_count(), 1);

        undo.modify(|_| ());
        assert_eq!(undo.undo_count(), 2);
    }

    #[test]
    fn redo_count() {
        let mut undo = UndoWrapper::new(());

        assert_eq!(undo.redo_count(), 0);

        undo.modify(|_| ());
        undo.modify(|_| ());
        assert_eq!(undo.redo_count(), 0);

        undo.undo();
        assert_eq!(undo.redo_count(), 1);

        undo.undo();
        assert_eq!(undo.redo_count(), 2);

        undo.redo();
        assert_eq!(undo.redo_count(), 1);

        undo.redo();
        assert_eq!(undo.redo_count(), 0);
    }

    #[test]
    fn edit_clears_redo_states() {
        let mut undo = UndoWrapper::new(0i32);
        assert_eq!(undo.state(), &0);

        undo.modify(|x| *x += 1);
        assert_eq!(undo.state(), &1);

        undo.modify(|x| *x += 1);
        assert_eq!(undo.state(), &2);

        undo.undo();
        assert_eq!(undo.state(), &1);

        undo.undo();
        assert_eq!(undo.state(), &0);

        // push a completely new value. the redo states should be cleared.
        undo.modify(|x| *x += 10);
        assert_eq!(undo.state(), &10);

        // doing redo now should not result in a previous value
        assert!(!undo.redo());
        assert_eq!(undo.state(), &10);
        assert!(!undo.redo());
        assert_eq!(undo.state(), &10);
    }

    #[test]
    fn invalid_undo() {
        let mut undo = UndoWrapper::new(());

        undo.modify(|_| ());

        assert!(undo.undo());
        assert!(!undo.undo());
        assert!(!undo.undo());
    }

    #[test]
    fn invalid_redo() {
        let mut undo = UndoWrapper::new(());

        undo.modify(|_| ());
        assert!(undo.undo());

        assert!(undo.redo());
        assert!(!undo.redo());
        assert!(!undo.redo());
    }

    #[test]
    fn can_undo_to_clean_state() {
        let mut undo = UndoWrapper::new(());
        assert!(undo.is_dirty());

        undo.mark_clean();
        assert!(!undo.is_dirty());

        undo.modify(|_| ());
        assert!(undo.is_dirty());

        undo.undo();
        assert!(!undo.is_dirty());

        undo.redo();
        assert!(undo.is_dirty());

        undo.undo();
        assert!(!undo.is_dirty());
    }

    #[test]
    fn edit_wipes_future_clean_state() {
        let mut undo = UndoWrapper::new(());
        assert!(undo.is_dirty());

        undo.modify(|_| ());
        undo.mark_clean();
        assert!(!undo.is_dirty());

        undo.undo();
        undo.modify(|_| ());
        assert!(undo.is_dirty());
    }
}
