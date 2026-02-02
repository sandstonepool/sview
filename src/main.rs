//! sview - A TUI for monitoring Cardano nodes
//!
//! This application connects to a Cardano node's Prometheus metrics endpoint
//! and displays real-time status information in a terminal user interface.
//!
//! Supports monitoring multiple nodes via config file (~/.config/sview/config.toml).

mod app;
mod config;
mod history;
mod metrics;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;

use app::{App, AppMode};
use config::AppConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration from CLI, environment, and config file
    let app_config = AppConfig::load();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state and run
    let mut app = App::new(app_config);
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
    // Initial metrics fetch for all nodes
    app.fetch_all_metrics().await;

    loop {
        // Draw UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle input with timeout for periodic refresh
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // In help mode, any key closes help
                    if app.mode == AppMode::Help {
                        app.toggle_help();
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('r') => app.fetch_all_metrics().await,
                        KeyCode::Char('?') => app.toggle_help(),
                        
                        // Node switching
                        KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            app.prev_node();
                        }
                        KeyCode::Tab => {
                            app.next_node();
                        }
                        KeyCode::BackTab => {
                            app.prev_node();
                        }
                        
                        // Number keys to select nodes directly (1-9)
                        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                            let index = (c as usize) - ('1' as usize);
                            app.select_node(index);
                        }
                        
                        // Left/Right arrow keys for node switching
                        KeyCode::Left => {
                            app.prev_node();
                        }
                        KeyCode::Right => {
                            app.next_node();
                        }
                        
                        _ => {}
                    }
                }
            }
        }

        // Periodic refresh
        app.tick().await;
    }
}
