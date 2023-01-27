use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::ui::Component;

pub struct ModalKey<T: Component> {
    index: usize,
    data: PhantomData<T>,
}

impl<T: Component> Clone for ModalKey<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            data: self.data,
        }
    }
}
impl<T: Component> Copy for ModalKey<T> {}

#[derive(Default)]
pub struct ModalCollection {
    modals: Vec<Box<dyn Component>>,
}

impl ModalCollection {
    // NOTE: could separate into mutable and immutable components, ie. a mutable builder and immutable collection
    pub fn insert<T: Component + 'static>(&mut self, item: T) -> ModalKey<T> {
        let my_box: Box<dyn Component> = Box::new(item);
        self.modals.push(my_box);
        ModalKey {
            index: self.modals.len() - 1,
            data: PhantomData::default(),
        }
    }
}

impl<TComponent: Component> Index<ModalKey<TComponent>> for ModalCollection {
    type Output = TComponent;

    fn index(&self, key: ModalKey<TComponent>) -> &Self::Output {
        self.modals[key.index]
            .downcast_ref()
            .expect("retrieve component")
    }
}

impl<TComponent: Component> IndexMut<ModalKey<TComponent>> for ModalCollection {
    fn index_mut(&mut self, key: ModalKey<TComponent>) -> &mut Self::Output {
        self.modals[key.index]
            .downcast_mut()
            .expect("retrieve component")
    }
}

impl Component for ModalCollection {
    fn pre_render(
        &self,
        global_state: &crate::ui::AppState,
        frame_storage: &mut crate::ui::FrameLocalStorage,
    ) {
        self.modals
            .iter()
            .for_each(|m| m.pre_render(global_state, frame_storage))
    }

    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        state: &crate::ui::AppState,
        frame_storage: &crate::ui::FrameLocalStorage,
    ) {
        self.modals
            .iter()
            .for_each(|m| m.render(frame, area, state, frame_storage))
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        state: &mut crate::ui::AppState,
        frame_storage: &crate::ui::FrameLocalStorage,
    ) -> bool {
        for modal in &mut self.modals {
            if modal.process_input(key, state, frame_storage) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use td_lib::database::Database;

    use super::*;
    use crate::ui::{AppState, Component};

    struct TestComponent;
    impl Component for TestComponent {
        fn pre_render(
            &self,
            _global_state: &crate::ui::AppState,
            _frame_storage: &mut crate::ui::FrameLocalStorage,
        ) {
        }

        fn render(
            &self,
            _frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
            _area: tui::layout::Rect,
            _state: &crate::ui::AppState,
            _frame_storage: &crate::ui::FrameLocalStorage,
        ) {
        }

        fn process_input(
            &mut self,
            _key: crossterm::event::KeyEvent,
            _state: &mut crate::ui::AppState,
            _frame_storage: &crate::ui::FrameLocalStorage,
        ) -> bool {
            false
        }
    }

    #[test]
    /// This test ensures that there are no downcast errors when getting modals
    /// by their concrete type
    pub fn retrieve_does_not_panic() {
        let mut collection = ModalCollection::default();
        let key = collection.insert(TestComponent);
        _ = &collection[key];
        _ = &mut collection[key];
    }

    #[test]
    /// This test ensures that there are no downcast errors when iterating over
    /// modals internally.
    pub fn component_methods_do_not_panic() {
        let mut collection = ModalCollection::default();
        _ = collection.insert(TestComponent);

        let mut app_state = AppState {
            database: Database::default(),
            path: PathBuf::new(),
        };
        let mut frame_storage = Default::default();

        collection.pre_render(&app_state, &mut frame_storage);
        // NOTE: render requires a Frame<CrosstermBackent<Stdout>>
        collection.process_input(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut app_state,
            &frame_storage,
        );
    }
}
