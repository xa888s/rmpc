use std::ops::{Deref, DerefMut};
use tui::widgets::ListState;

pub struct StatefulList<T> {
    items: Vec<T>,
    state: ListState,
}

impl<T: Deref<Target = str>> StatefulList<T> {
    pub fn new_with(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();

        // set first in list as selected
        if items.len() > 0 {
            state.select(Some(0));
        }
        StatefulList { items, state }
    }

    pub fn push(&mut self, item: T) {
        let old = self.state.selected();
        self.items.push(item);
        if self.items.len() > 0 {
            self.state.select(old);
        }
    }

    pub fn pop(&mut self) {
        let old = self.state.selected();
        self.items.pop();
        if self.items.len() > 0 {
            self.state.select(old);
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.state.selected().map(|i| &*self.items[i])
    }

    // Select the next item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // Select the previous item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // Unselect the currently selected item if any. The implementation of `ListState` makes
    // sure that the stored offset is also reset.
    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn as_parts(&mut self) -> (&Vec<T>, &mut ListState) {
        (&self.items, &mut self.state)
    }
}

impl<T> Deref for StatefulList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> DerefMut for StatefulList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}
