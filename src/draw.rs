use crate::{play::Songs, state::StatefulList};
use mpd::Song;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    terminal::Frame,
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Wrap},
};

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
    events: &mut StatefulList<Songs, Song>,
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
    f.render_stateful_widget(list, chunk, events.state());
}

pub fn gauge<'a>(
    events: &mut StatefulList<Songs, Song>,
    f: &mut Frame<'a, CrosstermBackend<io::Stdout>>,
    chunk: Rect,
) {
    if let Some((song, status)) = events.song() {
        let (elapsed, duration) = match (status.elapsed, status.duration) {
            (Some(e), Some(d)) => (e.num_seconds() as f64, d.num_seconds() as f64),
            // should never really go here
            _ => (0., 1.),
        };

        let mut title = song.title.unwrap_or("Untitled".to_string()) + " ";
        title.insert_str(0, " ");
        let percent = (elapsed / duration) * 100.;
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent(percent as u16);
        f.render_widget(gauge, chunk);
    }
}

pub fn tags<'a>(
    events: &mut StatefulList<Songs, Song>,
    f: &mut Frame<'a, CrosstermBackend<io::Stdout>>,
    chunk: Rect,
) {
    if let Some(tags) = events.tags() {
        let tags = Paragraph::new(&*tags)
            .block(
                Block::default()
                    .title(" Tags ")
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Center);
        f.render_widget(tags, chunk);
    }
}

pub fn search<'a>(
    _events: &mut StatefulList<Songs, Song>,
    f: &mut Frame<'a, CrosstermBackend<io::Stdout>>,
    chunk: Rect,
    input: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1)])
        .split(chunk);

    let input = Paragraph::new(input)
        .block(
            Block::default()
                .title(" Search ")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, chunk);
    f.render_widget(input, chunks[0])
}

pub fn chunks<'a>(
    events: &mut StatefulList<Songs, Song>,
    f: &mut Frame<'a, CrosstermBackend<io::Stdout>>,
) -> DrawLayout {
    let term = f.size();

    let chunks = term
        .height
        .checked_sub(3)
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
                height: 3,
            };
            (songs, gauge)
        })
        .and_then(|(songs, gauge)| {
            events.tags().and_then(|tags| {
                let longest = (tags.split("\n").fold(0, |mut l, s| {
                    if l < s.len() {
                        l = s.len();
                    }
                    l
                 }) as u16)
                 // damn newlines taking up 2 bytes!
                     + 2;
                songs.width.checked_sub(longest).map(|list_size| {
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
                        width: longest,
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

fn search_box(frame_size: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - 10) / 2),
                Constraint::Percentage(10),
                Constraint::Percentage((100 - 10) / 2),
            ]
            .as_ref(),
        )
        .split(frame_size);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - 20) / 2),
                Constraint::Percentage(20),
                Constraint::Percentage((100 - 20) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
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
