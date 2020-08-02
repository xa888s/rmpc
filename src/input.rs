use crossterm::{
    event,
    event::Event,
    event::{KeyCode, KeyEvent},
};
use std::{
    sync::{mpsc, mpsc::Receiver, mpsc::Sender},
    thread,
    time::{Duration, Instant},
};

use crate::play;
use crate::state::StatefulList;
use mpd::Song;

pub fn get() -> Receiver<KeyEvent> {
    let (tx, rx) = mpsc::channel();

    let rate = Duration::from_millis(crate::TICK_RATE);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            if event::poll(rate - last_tick.elapsed()).unwrap() {
                if let Event::Key(k) = event::read().unwrap() {
                    tx.send(k).unwrap();
                }
            }
            if last_tick.elapsed() >= rate {
                last_tick = Instant::now();
            }
        }
    });
    rx
}

pub fn use_key(
    tx: &Sender<play::Message>,
    rx: &Receiver<play::Response>,
    events: &mut StatefulList<Vec<Song>, Song>,
    code: KeyCode,
) -> bool {
    let mut should_break = false;
    match code {
        KeyCode::Char(c) => match c {
            'q' => should_break = true,
            'j' => events.next(),
            'k' => events.previous(),
            'g' => events.select(0),
            'G' => events.select_last(),
            'd' => {
                if let Some(i) = events.selected_index() {
                    tx.send(play::Message::Delete(i)).unwrap();
                    if let play::Response::Songs(songs) = rx.recv().unwrap() {
                        events.set_songs(songs);
                    }
                }
            }
            // TODO: fix toggle pause not working
            'p' => {
                tx.send(play::Message::TogglePause).unwrap();
            }
            _ => {}
        },
        // TODO: fix block and subsequent panic due to invalid message format
        KeyCode::Enter => {
            if let Some(s) = events.selected() {
                tx.send(play::Message::Play(s.clone())).unwrap();
                if let play::Response::Songs(songs) = rx.recv().unwrap() {
                    events.set_songs(songs);
                    events.select(0);
                }
            }
        }
        KeyCode::Esc => should_break = true,
        _ => {}
    }
    should_break
}
