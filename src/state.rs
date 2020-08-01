use mpd::Song;
use std::ops::Deref;
use tui::{
    text::Span,
    widgets::{List, ListItem, ListState},
};

#[derive(Debug)]
pub struct StatefulList<T, A>
where
    T: Deref<Target = [A]>,
{
    items: T,
    tag_strs: Vec<String>,
    state: ListState,
}

impl<T, A> StatefulList<T, A>
where
    T: Deref<Target = [A]>,
{
    // Unselect the currently selected item if any. The implementation of `ListState` makes
    // sure that the stored offset is also reset.
    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn state(&mut self) -> &mut ListState {
        &mut self.state
    }
}

impl StatefulList<Vec<Song>, Song> {
    pub fn list<'a>(&self) -> List<'a> {
        List::new(
            self.items
                .iter()
                .map(|s| ListItem::new(Span::raw(s.title.clone().unwrap())))
                .collect::<Vec<ListItem<'a>>>(),
        )
    }

    pub fn selected(&self) -> Option<&Song> {
        self.state.selected().map(|i| &self.items[i])
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn select(&mut self, index: usize) {
        self.state.select(Some(if index == 0 {
            index
        } else {
            self.items.len() % index
        }));
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

    pub fn new_with_songs(items: Vec<Song>) -> StatefulList<Vec<Song>, Song> {
        let mut events = StatefulList {
            items,
            state: ListState::default(),
            tag_strs: Vec::new(),
        };
        if !events.items.is_empty() {
            events.state.select(Some(0));
        }
        // events are fresh and not changed after this
        events.tag_strs = events
            .items
            .iter()
            .map(|s| {
                let mut buf = String::new();
                s.tags.iter().for_each(|(t, s)| {
                    buf.push_str(&*t);
                    buf.push_str(": ");
                    buf.push_str(&*s);
                    buf.push_str("\n");
                });
                buf
            })
            .collect();
        events
    }

    pub fn set(&mut self, items: Vec<Song>) {
        self.items = items;
        self.tag_strs = self
            .items
            .iter()
            .map(|s| {
                let mut buf = String::new();
                s.tags.iter().for_each(|(t, s)| {
                    buf.push_str(&*t);
                    buf.push_str(": ");
                    buf.push_str(&*s);
                    buf.push_str("\n");
                });
                buf
            })
            .collect();
    }

    pub fn get_tags(&self) -> &[String] {
        &self.tag_strs
    }
}
