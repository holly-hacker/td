use crossterm::event::{KeyCode, KeyModifiers};
use tui::widgets::{Block, Borders, Clear, List, ListItem, ListState};

use crate::{
    ui::{
        constants::{LIST_HIGHLIGHT_STYLE, LIST_STYLE, MIN_MODAL_WIDTH},
        input::TextBoxComponent,
        Component,
    },
    utils::RectExt,
};

pub struct ListSearchModal<TKey: Eq + Clone> {
    title: String,
    items: Option<Vec<(TKey, String)>>,
    search_box: TextBoxComponent,
    index: usize,
}

impl<TKey: Eq + Clone> ListSearchModal<TKey> {
    pub fn new(title: String) -> Self {
        Self {
            title,
            items: None,
            search_box: TextBoxComponent::default(),
            index: 0,
        }
    }

    pub fn is_open(&self) -> bool {
        self.items.is_some()
    }

    pub fn open(&mut self, items: Vec<(TKey, String)>) {
        self.items = Some(items);
        self.search_box = TextBoxComponent::new_focused().with_background(true);
        self.index = 0;
    }

    pub fn close(&mut self) -> Option<TKey> {
        let ret = self.get_seach_results().nth(self.index).cloned();
        self.items = None;
        ret.map(|x| x.0)
    }

    fn get_seach_results(&self) -> Box<dyn Iterator<Item = &(TKey, String)> + '_> {
        let search_query = self.search_box.text().to_lowercase();
        match &self.items {
            Some(x) => Box::new(
                x.iter()
                    .filter(move |(_, x)| x.to_lowercase().contains(&search_query)),
            ),
            None => Box::new([].into_iter()),
        }
    }
}

impl<TKey: Eq + Clone> Component for ListSearchModal<TKey> {
    fn render(
        &self,
        frame: &mut tui::Frame<tui::backend::CrosstermBackend<std::io::Stdout>>,
        area: tui::layout::Rect,
        state: &crate::ui::AppState,
    ) {
        let Some(items) = &self.items else {return;};

        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL);

        let filtered_items = self.get_seach_results().collect::<Vec<_>>();

        let (list, mut list_state) = {
            let list = List::new(
                filtered_items
                    .iter()
                    .map(|item| ListItem::new(item.1.clone()))
                    .collect::<Vec<_>>(),
            )
            .style(LIST_STYLE)
            .highlight_style(LIST_HIGHLIGHT_STYLE);

            let mut list_state = ListState::default();
            list_state.select((!items.is_empty()).then_some(self.index));

            (list, list_state)
        };

        let height_list = 10;
        let block_height = height_list + TextBoxComponent::HEIGHT + 2;
        let block_width = MIN_MODAL_WIDTH
            .max(self.search_box.text().len() as u16 + 1)
            .max(self.title.len() as u16)
            + 2;

        // put the block in the center of the area
        let block_area = area.center_rect(block_width, block_height);

        let block_area_inner = block.inner(block_area);

        frame.render_widget(Clear, block_area);
        frame.render_widget(block, block_area);

        self.search_box.render(
            frame,
            block_area_inner.take_y(TextBoxComponent::HEIGHT),
            state,
        );
        frame.render_stateful_widget(
            list,
            block_area_inner.skip_y(TextBoxComponent::HEIGHT),
            &mut list_state,
        );
    }

    fn update(&mut self, key: crossterm::event::KeyEvent, state: &mut crate::ui::AppState) -> bool {
        // always close with Esc
        if self.is_open() && key.code == KeyCode::Esc {
            self.close();
            return true;
        }

        let Some(_items) = &self.items else {return false;};
        let filtered_item_count = self.get_seach_results().count();

        // NOTE: could abstract list into a component and have consistent list navigation everywhere
        let list_handled = match (key.code, key.modifiers) {
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.index = self.index.saturating_sub(1);
                true
            }
            (KeyCode::Up, KeyModifiers::ALT) => {
                self.index = 0;
                true
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                if filtered_item_count != 0 && self.index < filtered_item_count - 1 {
                    self.index += 1;
                }
                true
            }
            (KeyCode::Down, KeyModifiers::ALT) => {
                if filtered_item_count != 0 {
                    self.index = filtered_item_count - 1;
                }
                true
            }
            _ => false,
        };

        if !list_handled {
            let search_updated = self.search_box.update(key, state);

            if search_updated {
                if filtered_item_count != 0 {
                    self.index = self.index.clamp(0, filtered_item_count - 1);
                }

                return true;
            }
        }

        false
    }
}
