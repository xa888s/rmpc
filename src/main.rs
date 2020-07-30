mod input;
mod state;

use anyhow::{Context, Result};
use state::StatefulList;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use std::io;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

type Term = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<()> {
    let mut term = start()?;
    term.hide_cursor()?;

    let input = input::get_input();

    term.clear()?;

    let mut events = StatefulList::new_with(vec!["Bruh".to_string(), "Whip".to_string()]);
    loop {
        let (items, state) = events.as_parts();
        draw(&mut term, items, state)?;
        match input.try_recv() {
            Ok(key) => match key.code {
                KeyCode::Char(c) => match c {
                    'q' => break,
                    'j' => events.next(),
                    'k' => events.previous(),
                    _ => {}
                },
                KeyCode::Enter => {
                    events.push("Dab".to_string());
                }
                KeyCode::Esc => break,
                _ => {}
            },
            Err(_) => {}
        }
    }
    end(term)
}

fn start() -> Result<Term> {
    let mut stdout = io::stdout();
    stdout
        .execute(EnterAlternateScreen)?
        .execute(EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend).context("Failed on TUI initialization")?)
}

fn end(mut term: Term) -> Result<()> {
    term.show_cursor()?;
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout
        .execute(LeaveAlternateScreen)?
        .execute(DisableMouseCapture)?;
    Ok(())
}

fn draw(term: &mut Term, items: &[String], state: &mut ListState) -> Result<()> {
    term.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(f.size());

        let items: Vec<ListItem> = items
            .iter()
            .map(|item| ListItem::new(item.as_str()))
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Stuff").borders(Borders::ALL))
            .highlight_symbol(">>");
        f.render_stateful_widget(list, chunks[0], state);

        let block = Block::default()
            .title("Box with other stuff")
            .borders(Borders::ALL);
        f.render_widget(block, chunks[1]);
    })
    .context("Error in rendering loop")
}
