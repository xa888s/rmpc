use crate::{play::Songs, state::StatefulList};
use mpd::Song;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    terminal::Frame,
    text::Span,
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Wrap},
};

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
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(tags, chunk);
    }
}

pub fn chunks<'a>(
    events: &mut StatefulList<Songs, Song>,
    f: &mut Frame<'a, CrosstermBackend<io::Stdout>>,
) -> (Vec<Rect>, Vec<Rect>) {
    let constraints = if events.is_empty() {
        [Constraint::Percentage(100)].as_ref()
    } else {
        [Constraint::Percentage(70), Constraint::Percentage(30)].as_ref()
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(f.size());

    let constraints = if events.is_song_empty() {
        [Constraint::Percentage(100)].as_ref()
    } else {
        [Constraint::Min(3), Constraint::Max(3)].as_ref()
    };

    let sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(chunks[0]);

    (chunks, sub_chunks)
}
