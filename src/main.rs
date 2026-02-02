//! sview - A TUI for monitoring Cardano nodes
//!
//! This application connects to a Cardano node's Prometheus metrics endpoint
//! and displays real-time status information in a terminal user interface.

mod app;
mod config;
mod metrics;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;

use app::App;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize configuration from environment
    let config = Config::from_env();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state and run
    let mut app = App::new(config);
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    // Initial metrics fetch
    app.fetch_metrics().await;

    loop {
        // Draw UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle input with timeout for periodic refresh
        if event::poll(Duration::from_millis(1000))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('r') => app.fetch_metrics().await,
                        _ => {}
                    }
                }
            }
        }

        // Periodic refresh
        app.tick().await;
    }
}
