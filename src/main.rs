//! sview - A TUI for monitoring Cardano nodes
//!
//! This application connects to a Cardano node's Prometheus metrics endpoint
//! and displays real-time status information in a terminal user interface.
//!
//! Supports monitoring multiple nodes via config file (~/.config/sview/config.toml).

mod alerts;
mod app;
mod config;
mod geoip;
mod history;
mod metrics;
mod peers;
mod sockets;
mod storage;
mod themes;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;

use app::{App, AppMode};
use config::AppConfig;
use storage::StorageManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for logging (respects RUST_LOG env var)
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    // Load configuration from CLI, environment, and config file
    let app_config = AppConfig::load();

    // Handle --export flag: export to CSV and exit
    if let Some(export_path) = &app_config.export_path {
        return export_metrics(&app_config, export_path);
    }

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

                    // In peer detail mode, handle specific keys
                    if app.mode == AppMode::PeerDetail {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.toggle_peers().await;
                            }
                            KeyCode::Backspace | KeyCode::Left | KeyCode::Char('p') => {
                                app.back_to_peer_list();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // In peers mode, handle specific keys
                    if app.mode == AppMode::Peers {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.toggle_peers().await;
                            }
                            KeyCode::Char('p') => {
                                app.toggle_peers().await;
                            }
                            KeyCode::Char('r') => app.refresh_peers().await,
                            KeyCode::Up | KeyCode::Char('k') => app.peer_list_up(20),
                            KeyCode::Down | KeyCode::Char('j') => app.peer_list_down(20),
                            KeyCode::Enter | KeyCode::Right => app.show_peer_detail(),
                            _ => {}
                        }
                        continue;
                    }

                    // In graphs mode, handle specific keys
                    if app.mode == AppMode::Graphs {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('g') => {
                                app.toggle_graphs();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('r') => app.fetch_all_metrics().await,
                        KeyCode::Char('?') => app.toggle_help(),
                        KeyCode::Char('t') => app.cycle_theme(),
                        KeyCode::Char('p') => app.toggle_peers().await,
                        KeyCode::Char('g') => app.toggle_graphs(),

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

/// Export historical metrics to CSV file
fn export_metrics(app_config: &AppConfig, export_path: &std::path::Path) -> Result<()> {
    use std::path::PathBuf;

    println!("Exporting metrics to: {}", export_path.display());

    let mut total_exported = 0;

    for node in &app_config.nodes {
        let storage = StorageManager::new(&node.name);

        // Generate output path - if multiple nodes, append node name
        let output_path = if app_config.nodes.len() > 1 {
            let stem = export_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("export");
            let ext = export_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("csv");
            let sanitized_name = node.name.replace(' ', "_").to_lowercase();
            let new_name = format!("{}_{}.{}", stem, sanitized_name, ext);
            export_path
                .parent()
                .map(|p| p.join(&new_name))
                .unwrap_or_else(|| PathBuf::from(&new_name))
        } else {
            export_path.to_path_buf()
        };

        match storage.export_to_csv(&output_path) {
            Ok(count) => {
                println!(
                    "  [{}] Exported {} snapshots to {}",
                    node.name,
                    count,
                    output_path.display()
                );
                total_exported += count;
            }
            Err(e) => {
                eprintln!("  [{}] Export failed: {}", node.name, e);
            }
        }
    }

    if total_exported == 0 {
        println!("No historical data found. Run sview to collect metrics first.");
    } else {
        println!("Total: {} snapshots exported", total_exported);
    }

    Ok(())
}
