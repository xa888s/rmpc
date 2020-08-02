use anyhow::{Context, Result};
use mpd::{idle::Idle, Client, Song, Status, Subsystem};
use std::{
    net::ToSocketAddrs,
    sync::mpsc,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};

pub fn start_client<'a>(ip: impl ToSocketAddrs) -> Result<(Sender<Message>, Receiver<Response>)> {
    let mut conn = Client::connect(ip).context("Failed to connect to MPD server")?;

    let (message_tx, rx) = mpsc::channel();
    let (tx, response_rx) = mpsc::channel();

    thread::spawn(move || loop {
        match rx.try_recv() {
            Ok(m) => match m {
                Message::Play(s) => play(&mut conn, s).unwrap(),
                Message::Delete(i) => conn.delete(i as u32).unwrap(),
                Message::TogglePause => conn.toggle_pause().unwrap(),

                Message::Refresh => update(&mut conn, &tx),
            },
            Err(e) => match e {
                TryRecvError::Empty => {
                    let guard = conn.idle(&[]).unwrap();
                    if let Ok(messages) = guard.get() {
                        for m in messages {
                            if let Subsystem::Queue = m {
                                update(&mut conn, &tx);
                            }
                        }
                    }
                }
                TryRecvError::Disconnected => break,
            },
        }
        thread::sleep(Duration::from_millis(crate::TICK_RATE));
    });
    Ok((message_tx, response_rx))
}

fn update(conn: &mut Client, tx: &Sender<Response>) {
    send_updated_songs(conn, tx);
    send_current_song(conn, tx);
}

fn send_current_song(conn: &mut Client, tx: &Sender<Response>) {
    let current_song = conn.currentsong().unwrap();
    let status = conn.status().unwrap();
    tx.send(Response::Song {
        current_song,
        status,
    })
    .unwrap();
}

fn send_updated_songs(conn: &mut Client, tx: &Sender<Response>) {
    let songs = Response::Songs(conn.queue().unwrap());
    tx.send(songs).unwrap();
}

fn play(conn: &mut Client, s: Song) -> Result<()> {
    let songs = conn.queue()?;
    conn.pause(true)?;
    for song in songs.iter().filter(|song| **song == s) {
        conn.delete(song.place.unwrap().pos)?;
    }
    conn.insert(s, 0)?;
    conn.switch(0)?;
    conn.pause(false)?;
    Ok(())
}

pub enum Message {
    Play(Song),
    Delete(usize),
    TogglePause,
    Refresh,
}

#[non_exhaustive]
pub enum Response {
    Songs(Vec<Song>),
    Song {
        current_song: Option<Song>,
        status: Status,
    },
    Phantom,
}
