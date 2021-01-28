use crossterm::event::KeyCode;

use anyhow::Result;

use crate::{
    mpd::{play::Songs, search::Search},
    state::StatefulList,
    Mode,
};
use async_mpd::MpdClient;

pub async fn use_key(
    client: &mut MpdClient,
    list: &mut StatefulList<Songs>,
    results: &mut StatefulList<Songs>,
    srch: &mut Search,
    mode: &mut Mode,
    code: KeyCode,
) -> Result<Status> {
    if let Mode::Searching = mode {
        match code {
            KeyCode::Char(c) => {
                srch.push(c);
                srch.search(client).await?;
                results.set_songs(srch.results());
            }
            KeyCode::Enter | KeyCode::Tab => {
                if !results.is_empty() {
                    results.next();
                    *mode = Mode::Selecting;
                }
            }
            KeyCode::Backspace => {
                srch.pop();
                srch.search(client).await?;
                results.set_songs(srch.results());
            }
            KeyCode::Esc => *mode = Mode::Browsing,
            _ => {}
        }
    } else if let Mode::Selecting = mode {
        match code {
            KeyCode::Char(c) => match c {
                'j' => results.next(),
                'k' => results.previous(),
                'g' => results.select(0),
                'G' => results.select_last(),
                _ => {}
            },
            KeyCode::Up | KeyCode::BackTab => results.previous(),
            KeyCode::Down | KeyCode::Tab => results.next(),
            KeyCode::Enter => {
                if let Some(s) = results.selected() {
                    client.queue_add(&s.file).await?;
                    let id = client.queue().await?.last().map(|s| s.id).flatten();
                    if let Some(id) = id {
                        client.playid(id).await?;
                        srch.clear();
                        results.clear()
                    }
                    *mode = Mode::Browsing;
                }
            }
            KeyCode::Esc => *mode = Mode::Searching,
            _ => {}
        }
    } else {
        match code {
            KeyCode::Char(c) => match c {
                'q' => return Ok(Status::Break),
                'j' => list.next(),
                'k' => list.previous(),
                'g' => list.select(0),
                'G' => list.select_last(),
                'c' => client.queue_clear().await?,
                '/' => *mode = Mode::Searching,
                'p' => match client.status().await?.state.as_str() {
                    "pause" => client.play().await?,
                    _ => client.pause().await?,
                },
                _ => {}
            },
            KeyCode::Up | KeyCode::BackTab => list.previous(),
            KeyCode::Down | KeyCode::Tab => list.next(),
            KeyCode::Enter => {
                if let Some(s) = list.selected() {
                    if let Some(id) = s.id {
                        client.playid(id).await?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(Status::Continue)
}

pub enum Status {
    Continue,
    Break,
}
