use std::collections::VecDeque;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

const LOG_CAPACITY: usize = 10;

pub struct AppState {
    pub current_page: usize,
    pub total_pages: usize,
    pub pressed_key: Option<u8>,
    pub log: VecDeque<String>,
}

impl AppState {
    pub fn new(total_pages: usize) -> Self {
        Self {
            current_page: 0,
            total_pages,
            pressed_key: None,
            log: VecDeque::with_capacity(LOG_CAPACITY),
        }
    }

    pub fn push_log(&mut self, msg: String) {
        if self.log.len() == LOG_CAPACITY {
            self.log.pop_front();
        }
        self.log.push_back(msg);
    }
}

// ── rendering ──────────────────────────────────────────────────────────────

fn render(state: &AppState, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // title
        Constraint::Length(12), // button grid (3 rows × 4 lines)
        Constraint::Min(4),     // event log
    ])
    .split(frame.area());

    render_title(frame, chunks[0], state);
    render_buttons(frame, chunks[1], state);
    render_log(frame, chunks[2], state);
}

fn render_title(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = format!(
        " Vibe Keyboard    Page {} / {}    [q] quit ",
        state.current_page + 1,
        state.total_pages
    );
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::bordered()),
        area,
    );
}

fn render_buttons(frame: &mut Frame, area: Rect, state: &AppState) {
    // 3 rows of 5 buttons each; logical keys 1–15
    let rows = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(4),
    ])
    .split(area);

    for (row_idx, row_area) in rows.iter().enumerate() {
        let cols = Layout::horizontal([
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
        ])
        .split(*row_area);

        for (col_idx, col_area) in cols.iter().enumerate() {
            let key_num = (row_idx * 5 + col_idx + 1) as u8;
            render_button(frame, *col_area, key_num, state);
        }
    }
}

fn render_button(frame: &mut Frame, area: Rect, key: u8, state: &AppState) {
    let is_pressed = state.pressed_key == Some(key);
    let is_nav = key == 11 || key == 12;

    let label = match key {
        11 => "◄ back".to_string(),
        12 => "fwd ►".to_string(),
        n => format!("{n}"),
    };

    let style = if is_pressed {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else if is_nav {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let block = Block::bordered().style(style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Vertically center the label
    let vert = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(inner);

    frame.render_widget(
        Paragraph::new(label)
            .alignment(Alignment::Center)
            .style(style),
        vert[1],
    );
}

fn render_log(frame: &mut Frame, area: Rect, state: &AppState) {
    let lines: Vec<Line> = state.log.iter().map(|s| Line::from(s.as_str())).collect();
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" Events ")),
        area,
    );
}

// ── public entry point ─────────────────────────────────────────────────────

pub fn run(state: Arc<Mutex<AppState>>, shutdown: Arc<AtomicBool>) {
    if let Err(e) = run_inner(state, shutdown) {
        eprintln!("[tui] {e}");
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
            let s = state.lock().unwrap();
            terminal.draw(|f| render(&s, f))?;
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
