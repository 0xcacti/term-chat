use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use anyhow::Result;
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    style::Stylize,
    widgets::Paragraph,
};
use std::io::stderr;

fn main() -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let mut counter = 0;
    loop {
        terminal.draw(|f| {
            f.render_widget(Paragraph::new(format!("Counter: {counter}")), f.size());
        })?;
        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match key.code {
                        crossterm::event::KeyCode::Char('k') => counter += 1,
                        crossterm::event::KeyCode::Char('j') => counter -= 1,
                        crossterm::event::KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
        }
    }
    // shutdown down: reset terminal back to original state
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
