use anyhow::{Context, Result};
use crossterm::event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use std::io;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders};
use tui::Terminal;

fn main() -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .execute(EnterAlternateScreen)?
        .execute(EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed on TUI initialization")?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let block = Block::default().title("Block").borders(Borders::ALL);
                f.render_widget(block, size);
            })
            .context("Error in rendering loop")?;
        if poll(Duration::from_millis(500))? {
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Char(c) if c == 'q' => break,
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }
    terminal.show_cursor()?;
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout
        .execute(LeaveAlternateScreen)?
        .execute(DisableMouseCapture)?;
    Ok(())
}
