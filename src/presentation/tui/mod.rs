mod app_state;
mod render;

pub use app_state::AppState;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::error;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub fn run(state: Arc<Mutex<AppState>>, shutdown: Arc<AtomicBool>) {
    if let Err(e) = run_inner(state, shutdown) {
        error!("TUI error: {e}");
    }
}

fn run_inner(state: Arc<Mutex<AppState>>, shutdown: Arc<AtomicBool>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        {
            let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
            s.throbber_state.calc_next();
            terminal.draw(|f| render::render(&mut s, f))?;
        }

        if event::poll(Duration::from_millis(33))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                    shutdown.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
