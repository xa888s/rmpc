use crossterm::{
    event,
    event::Event,
    event::{KeyCode, KeyEvent},
};
use std::{
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;

use crate::{
    play::{Message, Songs},
    state::StatefulList,
    Mode,
};
use mpd::Song;

pub fn get() -> Receiver<KeyEvent> {
    let (tx, rx) = mpsc::channel();

    let rate = Duration::from_millis(crate::TICK_RATE);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            if let Ok(s) = event::poll(rate - last_tick.elapsed()) {
                if s {
                    if let Ok(event) = event::read() {
                        if let Event::Key(k) = event {
                            if let Err(_) = tx.send(k) {
                                log::warn!("Failed to send key event")
                            }
                        }
                    }
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
    tx: &Sender<Message>,
    events: &mut StatefulList<Songs, Song>,
    mode: &mut Mode,
    code: KeyCode,
) -> Result<bool> {
    let mut should_break = false;
    if let Mode::Searching(s) = mode {
        match code {
            KeyCode::Char(c) => {
                s.push(c);
            }
            KeyCode::Enter => {}
            KeyCode::Backspace => {
                s.pop();
            }
            KeyCode::Esc => {
                *mode = Mode::Browsing;
            }
            _ => {}
        }
    } else {
        match code {
            KeyCode::Char(c) => match c {
                'q' => should_break = true,
                'j' => events.next(),
                'k' => events.previous(),
                'g' => events.select(0),
                'G' => events.select_last(),
                'd' => {
                    if let Some(i) = events.selected_index() {
                        tx.send(Message::Delete(i))?;
                    }
                }
                '/' => *mode = Mode::Searching(String::new()),
                'p' => {
                    tx.send(Message::TogglePause)?;
                }
                _ => {}
            },
            KeyCode::Enter => {
                if let Mode::Browsing = mode {
                    if let Some(s) = events.selected() {
                        let song = s.clone();
                        tx.send(Message::Play(song))?;
                        events.select(0);
                    }
                }
            }
            KeyCode::Esc => {}
            _ => {}
        }
    }
    Ok(should_break)
}
