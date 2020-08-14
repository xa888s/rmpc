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

use env_logger;

use std::{io, net::SocketAddrV4, sync::mpsc::TryRecvError, thread, time::Duration};

use tui::{backend::CrosstermBackend, Terminal};

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "rmpc")]
struct Opt {
    #[structopt(short, long, default_value = "127.0.0.1:6600")]
    addr: SocketAddrV4,
}

#[derive(Clone)]
pub enum Mode {
    Browsing,
    Searching(String),
}

const TICK_RATE: u64 = 17;

type Term = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<()> {
    let opt = Opt::from_args();

    env_logger::init();
    let mut term = start()?;
    term.hide_cursor()?;

    let input = input::get();

    log::debug!("Starting MPD thread");
    let (tx, rx) = play::start_client(opt.addr)?;

    term.clear()?;

    log::debug!("Getting initial song info");
    tx.send(Message::Refresh)?;
    let songs: Songs = rx.try_recv().unwrap_or_default();

    let mut events = StatefulList::new_with_songs(songs);
    let mut mode: Mode = Mode::Browsing;

    loop {
        thread::sleep(Duration::from_millis(TICK_RATE));
        draw(&mut term, &mut events, &mut mode)?;

        // handling key events
        if let Ok(k) = input.try_recv() {
            if let Ok(b) = input::use_key(&tx, &mut events, &mut mode, k.code) {
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

fn draw(term: &mut Term, events: &mut StatefulList<Songs, Song>, mode: &mut Mode) -> Result<()> {
    term.draw(|f| {
        let chunks = draw::chunks(events, f);
        if let draw::DrawLayout::Normal {
            songs,
            gauge,
            search,
        } = chunks
        {
            let draw::Chunks { list, tags } = songs;
            draw::tags(events, f, tags);
            draw::gauge(events, f, gauge);
            draw::list(events, f, list);

            if let Mode::Searching(i) = mode {
                draw::search(events, f, search, &i);
            }
        } else if let draw::DrawLayout::Empty(songs, search) = chunks {
            draw::list(events, f, songs);
            if let Mode::Searching(i) = mode {
                draw::search(events, f, search, &i);
            }
        }
    })
    .context("Error in rendering loop")
}
