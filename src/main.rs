mod draw;
mod input;
mod play;
mod state;

use anyhow::{Context, Result};
use mpd::Song;
use play::{Message, Songs};
use state::StatefulList;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use log::Level;
use simple_logger;

use std::{io, sync::mpsc::TryRecvError, thread, time::Duration};

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

    tx.send(Message::Refresh)?;
    let songs: Songs = rx.try_recv().unwrap_or_default();

    let mut events = StatefulList::new_with_songs(songs);

    loop {
        thread::sleep(Duration::from_millis(TICK_RATE));
        draw(&mut term, &mut events)?;

        // handling key events
        if let Ok(k) = input.try_recv() {
            if let Ok(b) = input::use_key(&tx, &mut events, k.code) {
                if b {
                    break;
                }
            } else {
                log::error!("Failed to use user input");
            }
        }

        // handling update events
        match rx.try_recv() {
            Ok(songs) => events.set_items(songs),
            Err(e) => {
                if let TryRecvError::Disconnected = e {
                    panic!("MPD Thread shut down before end of term")
                }
            }
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

fn draw(term: &mut Term, events: &mut StatefulList<Songs, Song>) -> Result<()> {
    term.draw(|f| {
        let (chunks, sub_chunks) = draw::chunks(events, f);
        let list_chunk = match (chunks.len(), sub_chunks.len()) {
            (2, 2) => {
                draw::tags(events, f, chunks[1]);
                draw::gauge(events, f, sub_chunks[1]);
                sub_chunks[0]
            }
            (_, _) => chunks[0],
        };

        draw::list(events, f, list_chunk);
    })
    .context("Error in rendering loop")
}
