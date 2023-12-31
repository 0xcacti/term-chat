pub mod app;
use app::{App, CurrentScreen};
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use anyhow::Result;
use ratatui::{
    prelude::{Backend, CrosstermBackend, Terminal},
    style::Stylize,
    widgets::Paragraph,
};
use std::io::{self, stderr};

pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;

fn main() -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stderr(), crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stderr());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(do_print) = res {
        if do_print {
            app.print_json()?;
        }
    } else if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        if let Event::Key(key) = event::read()? {
            dbg!(key.code);
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }
            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('e') => {
                        app.current_screen = CurrentScreen::Edit;
                        app.currently_editing = Some(CurrentlyEditing::Key);
                    }
                    KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('y') => {
                        return Ok(true);
                    }
                    KeyCode::Char('n') => {
                        return Ok(false);
                    }
                    _ => {}
                },
                CurrentScreen::Editing => match key.code 

            }
        }
    }
}
