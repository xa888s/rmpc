use crate::{play::Songs, search::Search, state::StatefulList};
use async_mpd::Status;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    terminal::Frame,
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, LineGauge, Paragraph},
};

const SEARCH_BOX_HEIGHT: u16 = 3;

// smallest size of heigh/width before crossterm/tui panics
const MIN_SIZE: u16 = 2;

// Playing layout
//
//       Song list             Tags for selected song
//          \/                           \/
// /---------------------------------------------\
// |                      |                      |
// |                      |                      |
// |                      |                      |
// |                      |                      |
// |                      |                      |
// |______________________|______________________|
// | ---------------->                           |
// \_____________________________________________/
//            /\
//  Progress of current song

pub fn list<'a>(
    events: &mut StatefulList<Songs>,
    f: &mut Frame<'a, CrosstermBackend<io::Stdout>>,
    chunk: Rect,
) {
    let list = events
        .list()
        .block(
            Block::default()
                .title([Span::styled(" Songs ", Style::default().fg(Color::White))].to_vec())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::default().fg(Color::Magenta))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunk, &mut *events.state());
}

pub fn gauge<'a>(
    status: Option<&Status>,
    f: &mut Frame<'a, CrosstermBackend<io::Stdout>>,
    chunk: Rect,
) {
    if let Some(status) = status {
        let (elapsed, duration) = match (status.elapsed, status.duration) {
            (Some(e), Some(d)) => (e.as_secs_f64(), d.as_secs_f64()),
            // should never really go here
            _ => (0., 1.),
        };

        let ratio = elapsed / duration;
        let gauge = LineGauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .ratio(ratio);
        f.render_widget(gauge, chunk);
    } else {
        log::error!("Cannot get song status");
    }
}

pub fn tags(tags: Option<String>, f: &mut Frame<'_, CrosstermBackend<io::Stdout>>, chunk: Rect) {
    if let Some(tags) = tags {
        let tags = Paragraph::new(&*tags)
            .block(
                Block::default()
                    .title(" Tags ")
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Center);
        f.render_widget(tags, chunk);
    } else {
        log::error!("Cannot find tags for song");
    }
}

pub fn search<'a>(
    list: &mut StatefulList<Songs>,
    f: &mut Frame<'a, CrosstermBackend<io::Stdout>>,
    chunk: Rect,
    input: &Search,
) {
    let width = chunk.width;

    let search_box = Paragraph::new(input.get(width as usize))
        .block(
            Block::default()
                .title(" Search ")
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Reset))
                .borders(Borders::ALL),
        )
        .alignment(Alignment::Left);

    if list.is_empty() {
        let middle = chunk.height / 2;

        let search = Rect {
            x: chunk.x,
            y: middle.saturating_sub(1),
            // fixed size
            height: SEARCH_BOX_HEIGHT,
            width,
        };

        f.render_widget(Clear, search);
        f.render_widget(search_box, search);
    } else {
        let clear = Rect {
            x: chunk.x,
            y: 1,
            height: chunk.height.saturating_sub(5),
            width,
        };
        let search = Rect {
            x: chunk.x,
            y: 1,
            height: SEARCH_BOX_HEIGHT,
            width,
        };

        let results = Rect {
            x: chunk.x,
            y: SEARCH_BOX_HEIGHT + 1,
            height: chunk.height - SEARCH_BOX_HEIGHT - 5,
            width,
        };

        let results_box = list
            .list()
            .block(
                Block::default()
                    .title([Span::styled(" Songs ", Style::default().fg(Color::White))].to_vec())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().fg(Color::Magenta))
            .highlight_symbol(">> ");
        f.render_widget(Clear, clear);
        f.render_widget(search_box, search);
        f.render_stateful_widget(results_box, results, &mut *list.state());
    }
}

pub fn chunks<'a>(
    events: &mut StatefulList<Songs>,
    f: &mut Frame<'a, CrosstermBackend<io::Stdout>>,
) -> DrawLayout {
    let term = f.size();

    let chunks = term
        .height
        .checked_sub(SEARCH_BOX_HEIGHT)
        .map(|height| {
            let songs = Rect {
                x: term.x,
                y: term.y,
                width: term.width,
                height,
            };

            let gauge = Rect {
                x: term.x,
                y: height,
                width: term.width,
                // fixed height
                height: SEARCH_BOX_HEIGHT,
            };
            (songs, gauge)
        })
        .and_then(|(songs, gauge)| {
            events.tags().and_then(|tags| {
                let longest = (tags.split('\n').fold(0, |mut l, s| {
                    if l < s.len() {
                        l = s.len();
                    }
                    l
                 }) as u16)
                 // damn newlines taking up 2 bytes!
                     + 2;
                let width = match longest {
                    // min size
                    0..=25 => 25,
                    26..=35 => 35,
                    // max size
                    _ => 40,
                };
                songs.width.checked_sub(width).map(|list_size| {
                    // space the list takes up
                    let list = Rect {
                        x: songs.x,
                        y: songs.y,
                        width: list_size,
                        height: songs.height,
                    };

                    // space the tags take up
                    let tags = Rect {
                        x: list_size,
                        y: songs.y,
                        width,
                        height: songs.height,
                    };

                    (Chunks { list, tags }, gauge)
                })
            })
        });
    let search = search_box(term);
    if let Some((songs, gauge)) = chunks {
        DrawLayout::Normal {
            songs,
            gauge,
            search,
        }
    } else {
        DrawLayout::Empty(term, search)
    }
}

fn search_box(f: Rect) -> Rect {
    // only resize if the size changes by at least 10
    let cells = f.width % 10;
    let width = if cells >= MIN_SIZE {
        f.width - cells
    } else if cells < MIN_SIZE {
        f.width - (cells + 10)
    } else {
        5
    };

    let middle = f.width / 2;

    Rect {
        x: middle - (width / 2),
        y: 0,
        width,
        height: f.height,
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DrawLayout {
    Normal {
        songs: Chunks,
        gauge: Rect,
        search: Rect,
    },
    Empty(Rect, Rect),
}

#[derive(Debug, Copy, Clone)]
pub struct Chunks {
    pub list: Rect,
    pub tags: Rect,
}
