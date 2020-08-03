use anyhow::{Context, Result};
use mpd::{Client, Song, Status};
use std::{
    net::ToSocketAddrs,
    ops::{Deref, DerefMut, Index, IndexMut},
    sync::mpsc,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};

pub fn start_client<'a>(ip: impl ToSocketAddrs) -> Result<(Sender<Message>, Receiver<Songs>)> {
    let mut conn = Client::connect(ip).context("Failed to connect to MPD server")?;

    // Channels
    //
    // message_tx
    // main thread -> to this thread
    //
    // used to send MPD commands from this thread
    //
    // response_rx
    // this thread -> to main thread
    //
    // used to return results or other from MPD
    //

    let (message_tx, rx) = mpsc::channel();
    let (tx, response_rx) = mpsc::channel();

    thread::spawn(move || {
        // should update
        loop {
            let (mut cached_song, mut cached_songs): (Option<Song>, Option<Vec<Song>>) =
                (None, None);
            match rx.try_recv() {
                Ok(m) => match m {
                    Message::Play(s) => {
                        if let Err(_) = play(&mut conn, &s) {
                            log::warn!("Failed to play song");
                        }
                        cached_song = Some(s);
                    }
                    Message::Delete(i) => {
                        if let Err(_) = conn.delete(i as u32) {
                            log::warn!("Failed to delete song from queue");
                        }
                    }
                    Message::TogglePause => {
                        if let Err(_) = conn.toggle_pause() {
                            log::warn!("Failed to pause song");
                        }
                    }

                    Message::Refresh => {
                        if let Err(_) = update(&mut conn, &tx, None, None) {
                            break;
                        }
                    }
                },
                Err(e) => {
                    if let TryRecvError::Disconnected = e {
                        break;
                    }
                }
            }
            if let Err(_) = update(&mut conn, &tx, cached_songs, cached_song) {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
    Ok((message_tx, response_rx))
}

fn update(
    conn: &mut Client,
    tx: &Sender<Songs>,
    songs: Option<Vec<Song>>,
    song: Option<Song>,
) -> Result<()> {
    let song = match (song, conn.status()) {
        (Some(s), Ok(status)) => Some((s, status)),
        (None, Ok(status)) => match conn.currentsong() {
            Ok(Some(s)) => Some((s, status)),
            _ => None,
        },
        _ => None,
    };

    let songs = songs.unwrap_or_else(|| {
        if let Ok(q) = conn.queue() {
            q
        } else {
            Vec::new()
        }
    });

    tx.send(Songs::new(songs, song))?;
    Ok(())
}

fn play(conn: &mut Client, s: &Song) -> Result<()> {
    let songs = conn.queue()?;
    conn.pause(true)?;
    for song in songs.iter().filter(|song| **song == *s) {
        if let Some(p) = song.place {
            conn.delete(p.pos)?;
        }
    }
    conn.insert(s, 0)?;
    conn.switch(0)?;
    conn.pause(false)?;
    Ok(())
}

pub enum Message {
    /// Used to play song
    Play(Song),

    /// Used to delete song from main queue
    Delete(usize),

    /// Used to toggle the playing status of the current song
    TogglePause,

    /// Used to get lastest data sent to thread
    Refresh,
}

#[derive(Debug, Clone, Default)]
pub struct Songs {
    songs: Vec<Song>,
    current_song: Option<PlayingSong>,
}

#[derive(Debug, Clone)]
struct PlayingSong {
    pub song: Song,
    pub status: Status,
}

impl Songs {
    pub fn new(songs: Vec<Song>, current_song: Option<(Song, Status)>) -> Songs {
        Songs {
            songs,
            current_song: current_song.map(|(song, status)| PlayingSong::new(song, status)),
        }
    }

    /// Will not set to None, rather the None signifies if you want to change it or not
    pub fn set(&mut self, songs: Option<Vec<Song>>, current_song: Option<(Song, Status)>) {
        self.current_song = current_song.map(|(song, status)| PlayingSong::new(song, status));
        if let Some(s) = songs {
            self.songs = s;
        }
    }

    pub fn song(&self) -> Option<(Song, Status)> {
        self.current_song.clone().map(|s| (s.song, s.status))
    }

    pub fn is_song_empty(&self) -> bool {
        self.current_song.is_none()
    }
}

impl Deref for Songs {
    type Target = Vec<Song>;
    fn deref(&self) -> &Self::Target {
        &self.songs
    }
}

impl DerefMut for Songs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.songs
    }
}

impl Index<usize> for Songs {
    type Output = Song;
    fn index(&self, index: usize) -> &Self::Output {
        &self.songs[index]
    }
}

impl IndexMut<usize> for Songs {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.songs[index]
    }
}

impl PlayingSong {
    pub fn new(song: Song, status: Status) -> PlayingSong {
        PlayingSong { song, status }
    }
}
