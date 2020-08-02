use anyhow::{Context, Result};
use mpd::{idle::Idle, Client, Song, Status, Subsystem};
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
                        play(&mut conn, &s).unwrap();
                        cached_song = Some(s);
                    }
                    Message::Delete(i) => conn.delete(i as u32).unwrap(),
                    Message::TogglePause => conn.toggle_pause().unwrap(),

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
    let song = Some((
        song.unwrap_or_else(|| conn.currentsong().unwrap().unwrap()),
        conn.status()?,
    ));

    let songs = songs.unwrap_or_else(|| conn.queue().unwrap());

    tx.send(Songs::new(songs, song))?;
    Ok(())
}

fn play(conn: &mut Client, s: &Song) -> Result<()> {
    let songs = conn.queue()?;
    conn.pause(true)?;
    for song in songs.iter().filter(|song| **song == *s) {
        conn.delete(song.place.unwrap().pos)?;
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
    tag_strs: Vec<String>,
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
            tag_strs: Songs::get_tags(&songs),
            songs,
            current_song: current_song.map(|(song, status)| PlayingSong::new(song, status)),
        }
    }

    /// Will not set to None, rather the None signifies if you want to change it or not
    pub fn set(&mut self, songs: Option<Vec<Song>>, current_song: Option<(Song, Status)>) {
        self.current_song = current_song.map(|(song, status)| PlayingSong::new(song, status));
        if let Some(s) = songs {
            self.songs = s;
            self.tag_strs = Self::get_tags(&self.songs);
        }
    }

    pub fn song(&self) -> Option<(Song, Status)> {
        self.current_song.clone().map(|s| (s.song, s.status))
    }

    pub fn is_song_empty(&self) -> bool {
        self.current_song.is_none()
    }

    pub fn tags(&self) -> &[String] {
        &self.tag_strs
    }

    pub fn get_tags(items: &[Song]) -> Vec<String> {
        items
            .iter()
            .map(|s| {
                let mut buf = String::new();
                s.tags.iter().take(s.tags.len() - 1).for_each(|(t, s)| {
                    buf.push_str(&*t);
                    buf.push_str(": ");
                    buf.push_str(&*s);
                    buf.push_str("\n");
                });
                if let Some((_, s)) = s.tags.iter().last() {
                    let length = s.parse::<f64>().unwrap() as u64;
                    let (minutes, seconds) = (length / 60, length % 60);
                    buf.push_str(
                        &(if seconds < 10 {
                            format!("Length: {}:{}{}", minutes, "0", seconds)
                        } else {
                            format!("Length: {}:{}", minutes, seconds)
                        }),
                    );
                }
                buf
            })
            .collect()
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
