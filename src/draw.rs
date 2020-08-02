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
        let elapsed = status.elapsed.unwrap().num_seconds() as f64;
        let duration = status.duration.unwrap().num_seconds() as f64;
        let percent = (if elapsed == 0. { 1. } else { elapsed } / duration) * 100.;
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(song.title.as_deref().unwrap())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .percent(percent as u16);
        f.render_widget(gauge, chunk);
    }
}

pub fn tags<'a>(
    events: &mut StatefulList<Songs, Song>,
    f: &mut Frame<'a, CrosstermBackend<io::Stdout>>,
    chunk: Rect,
) {
    if let Some(i) = events.selected_index() {
        if let Some(tag_list) = events.tags().get(i) {
            let paragraph = Paragraph::new(&**tag_list)
                .block(Block::default().title(" Song ").borders(Borders::ALL))
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center);
            f.render_widget(paragraph, chunk);
        }
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
        [Constraint::Percentage(90), Constraint::Percentage(10)].as_ref()
    };

    let sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(chunks[0]);

    (chunks, sub_chunks)
}
