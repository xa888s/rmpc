use mpd::Song;
use std::ops::Deref;
use tui::{
    text::Span,
    widgets::{List, ListItem, ListState},
};

#[derive(Debug)]
pub struct StatefulList<'a, T, A>
where
    T: Deref<Target = [A]> + 'a,
{
    items: T,
    list: Vec<ListItem<'a>>,
    state: ListState,
}

impl<'a, T, A> StatefulList<'a, T, A>
where
    T: Deref<Target = [A]> + 'a,
{
    // Unselect the currently selected item if any. The implementation of `ListState` makes
    // sure that the stored offset is also reset.
    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn list(&self) -> List<'a> {
        List::new(self.list.clone())
    }

    pub fn state(&mut self) -> &mut ListState {
        &mut self.state
    }
}

impl<'a> StatefulList<'a, Vec<Song>, Song> {
    pub fn selected(&self) -> Option<&Song> {
        self.state.selected().map(|i| &self.items[i])
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
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

    pub fn new_with_songs(items: Vec<Song>) -> StatefulList<'a, Vec<Song>, Song> {
        let mut events = StatefulList {
            items,
            state: ListState::default(),
            list: Vec::new(),
        };
        if !events.items.is_empty() {
            events.state.select(Some(0));
        }
        // events are fresh and not changed after this
        unsafe { events.update() };
        events
    }

    pub fn set(&mut self, items: Vec<Song>) {
        self.items = items;
        unsafe { self.update() };
    }

    unsafe fn update(&mut self) {
        self.list
            .resize_with(self.items.len(), || ListItem::new(""));

        assert_eq!(self.items.len(), self.list.len());

        // make sure there is enough space for
        for (song, list_item) in self.items.iter().zip(self.list.iter_mut()) {
            let title = song.title.as_deref().unwrap();

            // safe as long as items memory is not modified
            *list_item = ListItem::new(std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                title.as_ptr(),
                title.len(),
            )));
        }
    }
}
