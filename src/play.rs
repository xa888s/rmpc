use anyhow::{Context, Result};
use mpd::{Client, Song};
use std::{
    net::ToSocketAddrs,
    sync::mpsc,
    sync::mpsc::{Receiver, Sender},
    thread,
};

pub fn start_client<'a>(ip: impl ToSocketAddrs) -> Result<(Sender<Message>, Receiver<Response>)> {
    let mut conn = Client::connect(ip).context("Failed to connect to MPD server")?;

    let (message_tx, rx) = mpsc::channel();
    let (tx, response_rx) = mpsc::channel();

    thread::spawn(move || loop {
        match rx.recv() {
            Ok(m) => {
                let mut update = true;
                match m {
                    Message::Play(s) => {
                        conn.pause(true).unwrap();
                        conn.insert(s, 0).unwrap();
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
                        update = true;
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
}
