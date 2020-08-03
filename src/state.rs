use crate::play::Songs;
use mpd::Song;
use std::{
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::Iter,
    vec::IntoIter,
};
use tui::{
    style::{Color, Style},
    text::Span,
    widgets::{List, ListItem, ListState, Row, Table},
};

#[derive(Debug)]
pub struct StatefulList<T, A>
where
    T: Index<usize, Output = A> + IndexMut<usize, Output = A>,
{
    items: T,
    state: ListState,
}

impl<T, A> StatefulList<T, A>
where
    T: Index<usize, Output = A> + IndexMut<usize, Output = A>,
{
    pub fn set_items(&mut self, items: T) {
        self.items = items;
    }

    pub fn state(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn selected(&self) -> Option<&A> {
        self.state.selected().map(|i| &self.items[i])
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }
}

impl StatefulList<Songs, Song> {
    pub fn list<'a>(&self) -> List<'a> {
        List::new(
            self.items
                .iter()
                .map(|s| ListItem::new(Span::raw(s.title.clone().unwrap_or("Default".to_string()))))
                .collect::<Vec<ListItem<'a>>>(),
        )
    }

    pub fn tags(&self) -> Option<String> {
        self.state.selected().map(|i| {
            self.items[i]
                .tags
                .iter()
                .take(self.items[i].tags.len() - 1)
                .fold(String::new(), |mut tags, (t, s)| {
                    tags.push_str(&*t);
                    tags.push_str(": ");
                    tags.push_str(&*s);
                    tags.push_str("\n");
                    tags
                })
        })
    }

    pub fn new_with_songs(songs: Songs) -> StatefulList<Songs, Song> {
        let mut state = ListState::default();
        state.select(Some(0));

        StatefulList {
            items: songs,
            state,
        }
    }

    pub fn select(&mut self, index: usize) {
        self.state.select(Some(if index == 0 {
            index
        } else {
            self.items.len() % index
        }));
    }

    pub fn select_last(&mut self) {
        self.state.select(Some(self.items.len() - 1));
    }

    // Select the next item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn next(&mut self) {
        self.state.select(Some(
            self.state
                .selected()
                .map(|i| if i >= self.items.len() - 1 { 0 } else { i + 1 })
                .unwrap_or(0),
        ));
    }

    // Select the previous item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn previous(&mut self) {
        self.state.select(Some(
            self.state
                .selected()
                .map(|i| if i == 0 { self.items.len() - 1 } else { i - 1 })
                .unwrap_or(0),
        ));
    }
}

impl<T, A> Deref for StatefulList<T, A>
where
    T: Index<usize, Output = A> + IndexMut<usize, Output = A>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T, A> DerefMut for StatefulList<T, A>
where
    T: Index<usize, Output = A> + IndexMut<usize, Output = A>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}
