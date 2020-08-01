use crossterm::{event, event::Event, event::KeyEvent};
use std::{
    sync::{mpsc, mpsc::Receiver},
    thread,
    time::{Duration, Instant},
};

pub fn get_input() -> Receiver<KeyEvent> {
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
