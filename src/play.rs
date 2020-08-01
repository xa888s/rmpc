use anyhow::{Context, Result};
use mpd::{Client, Song};
use std::{
    net::ToSocketAddrs,
    sync::mpsc,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

pub fn start_client<'a>(ip: impl ToSocketAddrs) -> Result<(Sender<Message>, Receiver<Response>)> {
    let mut conn = Client::connect(ip).context("Failed to connect to MPD server")?;

    let (message_tx, rx) = mpsc::channel();
    let (tx, response_rx) = mpsc::channel();

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(crate::TICK_RATE));
        match rx.recv() {
            Ok(m) => {
                let mut update = true;
                match m {
                    Message::Play(s) => {
                        let songs = conn.queue().unwrap();
                        conn.pause(true).unwrap();
                        for song in songs.iter().filter(|song| **song == s) {
                            conn.delete(song.place.unwrap().pos).unwrap();
                        }
                        conn.insert(s, 0).unwrap();
                        conn.switch(0).unwrap();
                        conn.pause(false).unwrap();
                    }
                    Message::Start => update = true,
                    Message::Clear => {
                        conn.clear().unwrap();
                    }
                    Message::Delete(i) => {
                        conn.delete(i as u32).unwrap();
                    }
                    Message::TogglePause => {
                        conn.toggle_pause().unwrap();
                        update = false;
                    }
                }
                if update {
                    send_updated_songs(&mut conn, &tx);
                }
            }
            Err(_) => break,
        }
    });
    Ok((message_tx, response_rx))
}

fn send_updated_songs(conn: &mut Client, tx: &Sender<Response>) {
    let songs = Response::Songs(conn.queue().unwrap());
    tx.send(songs).unwrap();
}

pub enum Message {
    Play(Song),
    Delete(usize),
    TogglePause,
    Start,
    Clear,
}

#[non_exhaustive]
pub enum Response {
    Songs(Vec<Song>),
    Phantom,
}
