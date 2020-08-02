mod draw;
mod input;
mod play;
mod state;

use anyhow::{Context, Result};
use mpd::Song;
use state::StatefulList;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use log::Level;
use simple_logger;

use std::{io, thread, time::Duration};

use tui::{backend::CrosstermBackend, Terminal};

const TICK_RATE: u64 = 17;

type Term = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<()> {
    simple_logger::init_with_level(Level::Info)?;
    let mut term = start()?;
    term.hide_cursor()?;

    let input = input::get();

    let (tx, rx) = play::start_client("127.0.0.1:6600")?;

    term.clear()?;

    tx.send(play::Message::Refresh)?;
    let songs = if let play::Response::Songs(s) = rx.recv().unwrap() {
        s
    } else {
        Vec::new()
    };

    let mut events = StatefulList::new_with_songs(songs);

    if let play::Response::Song {
        current_song,
        status,
    } = rx.recv().unwrap()
    {
        if let Some(song) = current_song {
            events.set_current_song(song, status);
        }
    }

    loop {
        thread::sleep(Duration::from_millis(TICK_RATE));
        draw(&mut term, &mut events)?;

        // handling key events
        if let Ok(k) = input.try_recv() {
            if input::use_key(&tx, &rx, &mut events, k.code) {
                break;
            }
        }

        // handling update events
        match (rx.try_recv(), rx.try_recv()) {
            (Ok(songs), Ok(song)) => {
                if let play::Response::Songs(songs) = songs {
                    events.set_songs(songs);
                }

                if let play::Response::Song {
                    current_song,
                    status,
                } = song
                {
                    match current_song {
                        Some(song) => events.set_current_song(song, status),
                        None => {}
                    }
                }
            }
            // seems to be triggering this one often...
            (Ok(_), Err(_)) => {
                end(term)?;
                panic!("Bad last message format");
            }
            (Err(_), Ok(_)) => {
                end(term)?;
                panic!("Bad first message format");
            }
            (Err(_), Err(_)) => {}
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
        let (chunks, sub_chunks) = draw::chunks(events, f);

        draw::tags(events, f, chunks[1]);
        draw::list(events, f, sub_chunks[0]);
        draw::gauge(events, f, sub_chunks[1]);
    })
    .context("Error in rendering loop")
}
