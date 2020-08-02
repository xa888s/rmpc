mod input;
mod play;
mod state;

use anyhow::{Context, Result};
use mpd::Song;
use state::StatefulList;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use std::{io, thread, time::Duration};

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Terminal,
};

const TICK_RATE: u64 = 17;

type Term = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<()> {
    let mut term = start()?;
    term.hide_cursor()?;

    let input = input::get_input();
    let (tx, rx) = play::start_client("127.0.0.1:6600")?;

    term.clear()?;

    tx.send(play::Message::Refresh)?;
    let songs = if let play::Response::Songs(s) = rx.recv().unwrap() {
        s
    } else {
        Vec::new()
    };
    //let first_songs = if let play::Response::Songs(s) = rx.recv().unwrap() {
    //    s
    //} else {
    //    Vec::new()
    //};

    let mut events = StatefulList::new_with_songs(songs);
    loop {
        thread::sleep(Duration::from_millis(TICK_RATE));
        draw(&mut term, &mut events)?;
        match input.try_recv() {
            Ok(key) => match key.code {
                KeyCode::Char(c) => match c {
                    'q' => break,
                    'j' => events.next(),
                    'k' => events.previous(),
                    'g' => events.select(0),
                    'G' => events.select_last(),
                    'd' => match events.selected_index() {
                        Some(i) => {
                            tx.send(play::Message::Delete(i))?;
                            if let play::Response::Songs(songs) = rx.recv()? {
                                events.set(songs);
                            }
                        }
                        None => {}
                    },
                    'p' => {
                        tx.send(play::Message::TogglePause)?;
                    }
                    _ => {}
                },
                KeyCode::Enter => match events.selected() {
                    Some(s) => {
                        tx.send(play::Message::Play(s.clone()))?;
                        if let play::Response::Songs(songs) = rx.recv()? {
                            events.set(songs);
                            events.select(0);
                        }
                    }
                    None => {}
                },
                KeyCode::Esc => break,
                _ => {}
            },
            Err(_) => {}
        }
        match rx.try_recv() {
            Ok(s) => {
                if let play::Response::Songs(songs) = s {
                    events.set(songs)
                }
            }
            Err(_) => {}
        }
    }
    end(term)
}

fn start() -> Result<Term> {
    let mut stdout = io::stdout();
    stdout
        .execute(EnterAlternateScreen)?
        .execute(EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend).context("Failed on TUI initialization")?)
}

fn end(mut term: Term) -> Result<()> {
    term.show_cursor()?;
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout
        .execute(LeaveAlternateScreen)?
        .execute(DisableMouseCapture)?;
    Ok(())
}

fn draw(term: &mut Term, events: &mut StatefulList<Vec<Song>, Song>) -> Result<()> {
    term.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(f.size());

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
        f.render_stateful_widget(list, chunks[0], events.state());

        match events.selected_index() {
            Some(i) => match events.get_tags().get(i) {
                Some(tag_list) => {
                    let paragraph = Paragraph::new(&**tag_list)
                        .block(Block::default().title(" Song ").borders(Borders::ALL))
                        .wrap(Wrap { trim: true })
                        .alignment(Alignment::Center);
                    f.render_widget(paragraph, chunks[1]);
                }
                None => {}
            },
            None => {}
        }
    })
    .context("Error in rendering loop")
}
