use crossterm::event::KeyCode;

use anyhow::Result;

use crate::{play::Songs, search::Search, state::StatefulList, Mode};
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
                results.set_songs(srch.results().to_owned());
            }
            KeyCode::Enter => {
                results.next();
                *mode = Mode::Selecting;
            }
            KeyCode::Backspace => {
                srch.pop();
                srch.search(client).await?;
                results.set_songs(srch.results().to_owned());
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
            KeyCode::Enter => {
                if let Some(s) = results.selected() {
                    client.queue_add(&s.file).await?;
                    let id = client.queue().await?.last().map(|s| s.id).flatten();
                    if let Some(id) = id {
                        client.playid(id).await?;
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
                '/' => *mode = Mode::Searching,
                'p' => match client.status().await?.state.as_str() {
                    "pause" => client.play().await?,
                    "play" | _ => client.pause().await?,
                },
                _ => {}
            },
            KeyCode::Enter => {
                if let Some(s) = list.selected() {
                    if let Some(id) = s.id {
                        client.playid(id).await?;
                    }
                }
            }
            KeyCode::Esc | _ => {}
        }
    }
    Ok(Status::Continue)
}

pub enum Status {
    Continue,
    Break,
}
