mod input;

use anyhow::{Context, Result};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use std::io;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    terminal::Frame,
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};

use rand::{distributions::Alphanumeric, thread_rng, Rng};

type Term = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<()> {
    let mut term = start()?;
    term.hide_cursor()?;

    let input = input::get_input();

    term.clear()?;
    loop {
        term.draw(draw).context("Error in rendering loop")?;
        match input.try_recv() {
            Ok(key) => match key.code {
                KeyCode::Char(c) if c == 'q' => break,
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

fn draw(f: &mut Frame<'_, CrosstermBackend<io::Stdout>>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(f.size());

    let rng = thread_rng();

    let items: Vec<String> = (0..=10)
        .into_iter()
        .map(|_| rng.sample_iter(&Alphanumeric).take(20).collect::<String>())
        .collect();

    let items: Vec<ListItem> = items.iter().map(|s| ListItem::new(s.as_str())).collect();

    let list = List::new(items)
        .block(Block::default().title("Stuff").borders(Borders::ALL))
        .highlight_symbol(">>");
    f.render_widget(list, chunks[0]);
    let block = Block::default()
        .title("Box with other stuff")
        .borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
}
