use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::ui::Component;

pub struct CollectionKey<T: Component> {
    index: usize,
    data: PhantomData<T>,
}

impl<T: Component> Clone for CollectionKey<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            data: self.data,
        }
    }
}
impl<T: Component> Copy for CollectionKey<T> {}

/// A collection of components that operate independently from eachother.
#[derive(Default)]
pub struct ComponentCollection {
    components: Vec<Box<dyn Component>>,
}

impl ComponentCollection {
    // NOTE: could separate into mutable and immutable components, ie. a mutable builder and immutable collection
    pub fn insert<T: Component + 'static>(&mut self, item: T) -> CollectionKey<T> {
        let my_box: Box<dyn Component> = Box::new(item);
        self.components.push(my_box);
        CollectionKey {
            index: self.components.len() - 1,
            data: PhantomData::default(),
        }
    }
}

impl<TComponent: Component> Index<CollectionKey<TComponent>> for ComponentCollection {
    type Output = TComponent;

    fn index(&self, key: CollectionKey<TComponent>) -> &Self::Output {
        self.components[key.index]
            .downcast_ref()
            .expect("retrieve component")
    }
}

impl<TComponent: Component> IndexMut<CollectionKey<TComponent>> for ComponentCollection {
    fn index_mut(&mut self, key: CollectionKey<TComponent>) -> &mut Self::Output {
        self.components[key.index]
            .downcast_mut()
            .expect("retrieve component")
    }
}

impl Component for ComponentCollection {
    fn pre_render(
        &self,
        global_state: &crate::ui::AppState,
        frame_storage: &mut crate::ui::FrameLocalStorage,
    ) {
        self.components
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
        self.components
            .iter()
            .for_each(|m| m.render(frame, area, state, frame_storage))
    }

    fn process_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        state: &mut crate::ui::AppState,
        frame_storage: &crate::ui::FrameLocalStorage,
    ) -> bool {
        for component in &mut self.components {
            if component.process_input(key, state, frame_storage) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    /// This test ensures that there are no downcast errors when getting
    /// components by their concrete type.
    pub fn retrieve_does_not_panic() {
        let mut collection = ComponentCollection::default();
        let key = collection.insert(TestComponent);
        _ = &collection[key];
        _ = &mut collection[key];
    }

    #[test]
    /// This test ensures that there are no downcast errors when iterating over
    /// components internally.
    pub fn component_methods_do_not_panic() {
        let mut collection = ComponentCollection::default();
        _ = collection.insert(TestComponent);

        let mut app_state = AppState::default();
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
