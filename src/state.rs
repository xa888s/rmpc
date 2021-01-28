use async_mpd::Track;
use std::{
    cell::{RefCell, RefMut},
    ops::{Deref, DerefMut, Index, IndexMut},
};
use tui::{
    text::Span,
    widgets::{List, ListItem, ListState},
};

#[derive(Debug, Clone)]
pub struct StatefulList<T> {
    items: T,
    state: RefCell<ListState>,
}

impl<T, A> StatefulList<T>
where
    T: Index<usize, Output = A> + IndexMut<usize, Output = A>,
{
    pub fn state(&self) -> RefMut<ListState> {
        self.state.borrow_mut()
    }
}

impl<T> StatefulList<T>
where
    T: Deref<Target = Vec<Track>>,
{
    pub fn selected(&self) -> Option<&Track> {
        self.state
            .borrow()
            .selected()
            .map(|i| self.items.get(i))
            .flatten()
    }

    pub fn list(&self) -> List<'_> {
        List::new(
            self.items
                .iter()
                .map(|s| ListItem::new(Span::raw(s.title.as_deref().unwrap_or("Untitled"))))
                .collect::<Vec<ListItem<'_>>>(),
        )
    }

    pub fn tags(&self) -> Option<String> {
        self.state.borrow().selected().and_then(|i| {
            self.items.get(i).map(|song| {
                let tags = [
                    ("Title:", &song.title),
                    ("Album:", &song.album),
                    ("Artist:", &song.artist),
                    ("Release Date:", &song.date),
                ];

                tags.iter()
                    .filter_map(|(n, t)| {
                        t.as_ref().map(|t| {
                            let mut buf = String::with_capacity(n.len() + t.len());
                            buf.push_str(n);
                            buf.push(' ');
                            buf.push_str(t);
                            buf.push('\n');
                            buf
                        })
                    })
                    .collect()
            })
        })
    }

    pub fn select(&mut self, index: usize) {
        self.state.borrow_mut().select(if self.items.len() != 0 {
            Some(if index == 0 {
                index
            } else {
                self.items.len() % index
            })
        } else {
            None
        });
    }

    pub fn select_last(&mut self) {
        self.state.borrow_mut().select(Some(self.items.len() - 1));
    }

    // Select the next item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn next(&mut self) {
        let index = Some({
            self.state
                .borrow()
                .selected()
                .map(|i| if i >= self.items.len() - 1 { 0 } else { i + 1 })
                .unwrap_or(0)
        });
        self.state.borrow_mut().select(index);
    }

    // Select the previous item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn previous(&mut self) {
        let index = Some({
            self.state
                .borrow()
                .selected()
                .map(|i| if i == 0 { self.items.len() - 1 } else { i - 1 })
                .unwrap_or(0)
        });

        self.state.borrow_mut().select(index);
    }
}

impl<T> Default for StatefulList<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            items: T::default(),
            state: RefCell::new(ListState::default()),
        }
    }
}

impl<T, A> Deref for StatefulList<T>
where
    T: Index<usize, Output = A> + IndexMut<usize, Output = A>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T, A> DerefMut for StatefulList<T>
where
    T: Index<usize, Output = A> + IndexMut<usize, Output = A>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}
