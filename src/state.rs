use mpd::{Song, Status};
use std::ops::Deref;
use tui::{
    text::Span,
    widgets::{List, ListItem, ListState},
};

#[derive(Debug, Clone)]
struct PlayingSong {
    pub current_song: Song,
    pub status: Status,
}

#[derive(Debug)]
pub struct StatefulList<T, A>
where
    T: Deref<Target = [A]>,
{
    items: T,
    song: Option<PlayingSong>,
    tag_strs: Vec<String>,
    state: ListState,
}

impl<T, A> StatefulList<T, A>
where
    T: Deref<Target = [A]>,
{
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

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
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

    pub fn select_last(&mut self) {
        self.state.select(Some(self.items.len() - 1));
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
            song: None,
            state: ListState::default(),
            tag_strs: Vec::new(),
        };
        if !events.items.is_empty() {
            events.state.select(Some(0));
        }

        events.tag_strs = Self::set_tags(&events.items);
        events
    }

    fn set_tags(items: &[Song]) -> Vec<String> {
        items
            .iter()
            .map(|s| {
                let mut buf = String::new();
                s.tags.iter().take(s.tags.len() - 1).for_each(|(t, s)| {
                    buf.push_str(&*t);
                    buf.push_str(": ");
                    buf.push_str(&*s);
                    buf.push_str("\n");
                });
                match s.tags.iter().last() {
                    Some((_, s)) => {
                        let length = s.parse::<f64>().unwrap() as u64;
                        let (minutes, seconds) = (length / 60, length % 60);
                        buf.push_str(
                            &(if seconds < 10 {
                                format!("Length: {}:{}{}", minutes, "0", seconds)
                            } else {
                                format!("Length: {}:{}", minutes, seconds)
                            }),
                        );
                    }
                    None => {}
                }
                buf
            })
            .collect()
    }

    pub fn set_songs(&mut self, items: Vec<Song>) {
        let old_i = self.state.selected();
        self.state.select(if items.len() == 0 {
            None
        } else if items.len() > self.items.len() {
            old_i
        } else {
            Some(0)
        });
        self.items = items;
        if self.items.len() > 0 {
            self.tag_strs = Self::set_tags(&self.items);
        }
    }

    pub fn set_current_song(&mut self, song: Song, status: Status) {
        self.song = Some(PlayingSong {
            current_song: song,
            status,
        });
    }

    pub fn current_song(&self) -> Option<(Song, Status)> {
        self.song.clone().map(|s| (s.current_song, s.status))
    }

    pub fn is_current_song_empty(&self) -> bool {
        self.song.is_none()
    }

    pub fn get_tags(&self) -> &[String] {
        &self.tag_strs
    }
}
