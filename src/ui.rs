//! User interface rendering
//!
//! This module handles all TUI rendering using ratatui.

use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

/// Main draw function - renders the entire UI
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Footer/status
        ])
        .split(area);

    // Draw header
    draw_header(frame, chunks[0], app);

    // Draw main content
    draw_main(frame, chunks[1], app);

    // Draw footer
    draw_footer(frame, chunks[2], app);
}

/// Draw the header section with node name and status
fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let status_indicator = if app.metrics.connected { "●" } else { "○" };
    let status_color = if app.metrics.connected {
        Color::Green
    } else {
        Color::Red
    };

    let header_text = Line::from(vec![
        Span::styled(
            format!(" {} ", app.config.node_name),
            Style::default().bold(),
        ),
        Span::styled(status_indicator, Style::default().fg(status_color)),
        Span::raw(" "),
        Span::styled(
            app.status_text(),
            Style::default().fg(status_color).italic(),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Network: {}", app.config.network),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Node: {}", app.metrics.node_type),
            Style::default().fg(Color::Yellow),
        ),
    ]);

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title(" sview "));

    frame.render_widget(header, area);
}

/// Draw the main content area with metrics
fn draw_main(frame: &mut Frame, area: Rect, app: &App) {
    // Split into left and right panels
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left panel: Chain metrics
    draw_chain_metrics(frame, chunks[0], app);

    // Right panel: Resource metrics
    draw_resource_metrics(frame, chunks[1], app);
}

/// Draw chain/block metrics
fn draw_chain_metrics(frame: &mut Frame, area: Rect, app: &App) {
    let metrics = &app.metrics;

    let rows = vec![
        Row::new(vec![
            Cell::from("Block Height"),
            Cell::from(format_metric_u64(metrics.block_height)),
        ]),
        Row::new(vec![
            Cell::from("Slot"),
            Cell::from(format_metric_u64(metrics.slot_num)),
        ]),
        Row::new(vec![
            Cell::from("Epoch"),
            Cell::from(format_metric_u64(metrics.epoch)),
        ]),
        Row::new(vec![
            Cell::from("Slot in Epoch"),
            Cell::from(format_metric_u64(metrics.slot_in_epoch)),
        ]),
        Row::new(vec![
            Cell::from("Sync Progress"),
            Cell::from(format_sync_progress(metrics.sync_progress)),
        ]),
        Row::new(vec![
            Cell::from("Connected Peers"),
            Cell::from(format_metric_u64(metrics.peers_connected)),
        ]),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Chain Status "),
    )
    .style(Style::default());

    frame.render_widget(table, area);
}

/// Draw resource/system metrics
fn draw_resource_metrics(frame: &mut Frame, area: Rect, app: &App) {
    let metrics = &app.metrics;

    let rows = vec![
        Row::new(vec![
            Cell::from("Memory Used"),
            Cell::from(format_bytes(metrics.memory_used)),
        ]),
        Row::new(vec![
            Cell::from("CPU Time"),
            Cell::from(format_duration_secs(metrics.cpu_seconds)),
        ]),
        Row::new(vec![
            Cell::from("Mempool TXs"),
            Cell::from(format_metric_u64(metrics.mempool_txs)),
        ]),
        Row::new(vec![
            Cell::from("Mempool Size"),
            Cell::from(format_bytes(metrics.mempool_bytes)),
        ]),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Resources "),
    )
    .style(Style::default());

    frame.render_widget(table, area);
}

/// Draw the footer with help and status
fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let endpoint = format!("{}:{}", app.config.prom_host, app.config.prom_port);

    let footer_text = if let Some(ref error) = app.last_error {
        Line::from(vec![
            Span::styled(" Error: ", Style::default().fg(Color::Red).bold()),
            Span::styled(
                truncate_string(error, 60),
                Style::default().fg(Color::Red),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(" [q] ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit  "),
            Span::styled("[r] ", Style::default().fg(Color::Yellow)),
            Span::raw("Refresh  "),
            Span::raw("| "),
            Span::styled(
                format!("Endpoint: {}", endpoint),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    };

    let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

// ============================================================================
// Formatting helpers
// ============================================================================

fn format_metric_u64(value: Option<u64>) -> String {
    value
        .map(|v| format_number(v))
        .unwrap_or_else(|| "—".to_string())
}

fn format_number(n: u64) -> String {
    // Add thousand separators
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_bytes(bytes: Option<u64>) -> String {
    match bytes {
        Some(b) if b >= 1_073_741_824 => format!("{:.2} GB", b as f64 / 1_073_741_824.0),
        Some(b) if b >= 1_048_576 => format!("{:.2} MB", b as f64 / 1_048_576.0),
        Some(b) if b >= 1024 => format!("{:.2} KB", b as f64 / 1024.0),
        Some(b) => format!("{} B", b),
        None => "—".to_string(),
    }
}

fn format_duration_secs(seconds: Option<f64>) -> String {
    match seconds {
        Some(s) if s >= 3600.0 => {
            let hours = s / 3600.0;
            format!("{:.1}h", hours)
        }
        Some(s) if s >= 60.0 => {
            let mins = s / 60.0;
            format!("{:.1}m", mins)
        }
        Some(s) => format!("{:.1}s", s),
        None => "—".to_string(),
    }
}

fn format_sync_progress(progress: Option<f64>) -> String {
    match progress {
        Some(p) => format!("{:.2}%", p),
        None => "—".to_string(),
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
