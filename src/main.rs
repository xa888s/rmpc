mod draw;
mod input;
mod play;
mod search;
mod state;

use anyhow::{Context, Result};
use play::Songs;
use state::StatefulList;

use search::Search;

use async_mpd::{Error, MpdClient, Subsystem};
use async_std::{channel, prelude::*, stream, task};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use directories_next as dirs;

use std::{
    io,
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use tui::{backend::CrosstermBackend, Terminal};

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "rmpc")]
struct Opt {
    #[structopt(short, long, default_value = "127.0.0.1")]
    ip: Ipv4Addr,

    #[structopt(short, long, default_value = "6600")]
    port: u16,
}

pub enum Mode {
    Browsing,
    Selecting,
    Searching,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Browsing
    }
}

type Term = Terminal<CrosstermBackend<io::Stdout>>;

struct App {
    song_list: StatefulList<Songs>,
    results: StatefulList<Songs>,
    search: Search,
    mode: Mode,
    term: Term,
}

impl App {
    pub fn new() -> Result<App> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut term = Terminal::new(backend).context("Failed on TUI initialization")?;

        enable_raw_mode()?;
        term.backend_mut()
            .execute(EnterAlternateScreen)?
            .execute(EnableMouseCapture)?;
        term.clear()?;
        Ok(App {
            term,
            search: Default::default(),
            mode: Default::default(),
            song_list: Default::default(),
            results: Default::default(),
        })
    }

    fn end(&mut self) -> Result<()> {
        self.term.show_cursor()?;
        disable_raw_mode()?;
        self.term
            .backend_mut()
            .execute(LeaveAlternateScreen)?
            .execute(DisableMouseCapture)?;
        Ok(())
    }

    fn args() -> Opt {
        Opt::from_args()
    }

    pub async fn run(mut self) -> Result<()> {
        if let Some(dir) = dirs::ProjectDirs::from("org", "abyss", "rmpc") {
            let mut data = dir.data_dir().to_path_buf();
            std::fs::create_dir_all(&data)?;
            data.push("rmpc.log");
            simple_logging::log_to_file(data, log::LevelFilter::Info)?;
        }
        log::info!("Starting up");
        let opts = App::args();
        let addr = SocketAddrV4::new(opts.ip, opts.port);
        let mut client = MpdClient::new(addr).await.context("Failed to start MPD client. Is it started and on 127.0.0.1:6600 or the specified port/ip?")?;
        let mut event_listener = MpdClient::new(addr).await.context("Failed to start MPD client. Is it started and on 127.0.0.1:6600 or the specified port/ip?")?;

        // start at the beginning of list
        self.song_list.next();

        // initial state
        self.song_list.set_status(client.status().await.ok());
        self.song_list
            .set_songs(&client.queue().await.unwrap_or_default());

        self.draw().await?;

        // Listening to MPD events
        let (s, mut r) = channel::bounded(1);
        let s2 = s.clone();
        let s3 = s.clone();

        task::spawn(async move {
            while let Some(u) = event_listener.idle().await.ok().flatten() {
                s.send(EventMessage::Mpd(u)).await.unwrap();
            }
        });

        // Listening to term events
        let mut input = EventStream::new();

        task::spawn(async move {
            while let Some(u) = input.next().await.transpose().ok().flatten() {
                s2.send(EventMessage::Term(u)).await.unwrap();
            }
        });

        // Listening to gauge events
        let mut interval = stream::interval(Duration::from_millis(500));

        task::spawn(async move {
            while interval.next().await.is_some() {
                s3.send(EventMessage::Tick).await.unwrap();
            }
        });

        // handling all events
        while let Some(u) = r.next().await {
            match u {
                EventMessage::Term(e) => {
                    if let Event::Key(k) = e {
                        if let Ok(b) = input::use_key(
                            &mut client,
                            &mut self.song_list,
                            &mut self.results,
                            &mut self.search,
                            &mut self.mode,
                            k.code,
                        )
                        .await
                        {
                            self.draw().await?;
                            if let input::Status::Break = b {
                                self.end()?;
                                break;
                            }
                        }
                    } else if let Event::Resize(_, _) = e {
                        self.draw().await?;
                    }
                }
                EventMessage::Mpd(u) => {
                    self.draw().await?;
                    match u {
                        Subsystem::Player | Subsystem::Mixer => {
                            let status = match client.status().await {
                                Ok(s) => Some(s),
                                Err(Error::Disconnected) => {
                                    client.reconnect().await?;
                                    None
                                }
                                _ => None,
                            };
                            self.song_list.set_status(status);
                        }
                        Subsystem::Playlist | Subsystem::StoredPlaylist => {
                            let queue = client.queue().await;
                            self.song_list
                                .set_songs(&queue.context("Can't set songs from update")?);
                        }
                        _ => {}
                    }
                }
                EventMessage::Tick => {
                    if let Some(u) = self.song_list.status() {
                        if u.state.as_str() == "play" {
                            self.song_list.set_status(client.status().await.ok());
                            self.draw().await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn draw(&mut self) -> Result<()> {
        let term = &mut self.term;
        let song_list = &mut self.song_list;
        let results = &mut self.results;
        let srch = &mut self.search;
        let mode = &self.mode;

        term.draw(|f| {
            let chunks = draw::chunks(song_list, f);
            let search = match &chunks {
                draw::DrawLayout::Normal { search, .. } | draw::DrawLayout::Empty(_, search) => {
                    search
                }
            };

            if let draw::DrawLayout::Normal {
                songs,
                gauge,
                search,
            } = &chunks
            {
                let draw::Chunks { list, tags } = songs;
                draw::tags(song_list.tags(), f, *tags);
                draw::gauge(song_list.status(), f, *gauge);
                draw::list(song_list, f, *list);
                if let Mode::Searching | Mode::Selecting = mode {
                    if f.size().height >= 3 {
                        draw::search(results, f, *search, srch);
                    }
                }
            } else if let draw::DrawLayout::Empty(songs, search) = &chunks {
                draw::list(song_list, f, *songs);
                if let Mode::Searching | Mode::Selecting = mode {
                    if f.size().height >= 3 {
                        draw::search(results, f, *search, srch);
                    }
                }
            }
            if let Mode::Searching | Mode::Selecting = mode {
                let search_box = srch.get(search.width as usize);

                let columns = ((search.x + 1) as usize + search_box.len()) as u16;
                let rows = search.height / 2;

                if !results.is_empty() {
                    match mode {
                        Mode::Selecting => {}
                        Mode::Searching => f.set_cursor(columns, 2),
                        _ => f.set_cursor(columns, 2),
                    }
                } else {
                    f.set_cursor(columns, rows);
                }
            }
        })
        .context("Error in rendering loop")?;

        Ok(())
    }
}

// Have to use an enum to combine all the streams :P
enum EventMessage {
    Term(Event),
    Mpd(Subsystem),
    Tick,
}

#[async_std::main]
async fn main() -> Result<()> {
    let app = App::new()?;
    app.run().await?;

    Ok(())
}
