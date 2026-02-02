//! User interface rendering
//!
//! This module handles all TUI rendering using ratatui.

use crate::app::{App, AppMode, HealthStatus};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Sparkline, Table, Tabs, Wrap},
};

/// Main draw function - renders the entire UI
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Create main layout - add node tabs if multi-node mode
    let chunks = if app.is_multi_node() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Node tabs
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Footer/status
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Footer/status
            ])
            .split(area)
    };

    // Draw node tabs if multi-node mode
    let (header_area, main_area, footer_area) = if app.is_multi_node() {
        draw_node_tabs(frame, chunks[0], app);
        (chunks[1], chunks[2], chunks[3])
    } else {
        (chunks[0], chunks[1], chunks[2])
    };

    // Draw header
    draw_header(frame, header_area, app);

    // Draw main content
    draw_main(frame, main_area, app);

    // Draw footer
    draw_footer(frame, footer_area, app);

    // Draw help overlay if in help mode
    if app.mode == AppMode::Help {
        draw_help_popup(frame, area, app.is_multi_node());
    }
}

/// Draw the node selection tabs
fn draw_node_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line> = app
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let health_color = health_to_color(node.overall_health());
            let indicator = if node.metrics.connected { "●" } else { "○" };
            let role_suffix = match node.role {
                crate::config::NodeRole::Bp => " [BP]",
                crate::config::NodeRole::Relay => "",
            };
            Line::from(vec![
                Span::styled(indicator, Style::default().fg(health_color)),
                Span::raw(" "),
                Span::raw(format!("{}{}", node.config.node_name, role_suffix)),
                Span::styled(
                    format!(" [{}]", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Nodes "))
        .select(app.selected_node)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" │ ");

    frame.render_widget(tabs, area);
}

/// Draw the header section with node name and status
fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let node = app.current_node();
    let health_color = health_to_color(node.overall_health());
    let status_indicator = if node.metrics.connected { "●" } else { "○" };

    let role_badge = match node.role {
        crate::config::NodeRole::Bp => Span::styled(
            " [BP] ",
            Style::default().fg(Color::Magenta).bold(),
        ),
        crate::config::NodeRole::Relay => Span::raw(""),
    };

    let header_text = Line::from(vec![
        Span::styled(
            format!(" {} ", node.config.node_name),
            Style::default().bold(),
        ),
        role_badge,
        Span::styled(status_indicator, Style::default().fg(health_color)),
        Span::raw(" "),
        Span::styled(
            node.status_text(),
            Style::default().fg(health_color).italic(),
        ),
        Span::raw(" │ "),
        Span::styled(
            format!("Network: {}", node.config.network),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" │ "),
        Span::styled(
            format!("Node: {}", node.metrics.node_type),
            Style::default().fg(Color::Yellow),
        ),
    ]);

    let header =
        Paragraph::new(header_text).block(Block::default().borders(Borders::ALL).title(" sview "));

    frame.render_widget(header, area);
}

/// Draw the main content area with metrics
fn draw_main(frame: &mut Frame, area: Rect, app: &App) {
    // Split into left and right panels
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left panel: Chain metrics with sparklines
    draw_chain_panel(frame, chunks[0], app);

    // Right panel: Resource metrics
    draw_resource_panel(frame, chunks[1], app);
}

/// Draw chain/block metrics panel
fn draw_chain_panel(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // Chain metrics table
            Constraint::Length(3), // Epoch progress gauge
            Constraint::Min(4),    // Block height sparkline
        ])
        .split(area);

    draw_chain_metrics(frame, chunks[0], app);
    draw_epoch_progress(frame, chunks[1], app);
    draw_block_sparkline(frame, chunks[2], app);
}

/// Draw chain metrics table
fn draw_chain_metrics(frame: &mut Frame, area: Rect, app: &App) {
    let node = app.current_node();
    let metrics = &node.metrics;
    let sync_health = node.sync_health();
    let peer_health = node.peer_health();
    let kes_health = node.kes_health();
    let tip_health = node.tip_health();

    let mut rows = vec![
        Row::new(vec![
            Cell::from("Block Height"),
            Cell::from(format_metric_u64(metrics.block_height)),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "Tip Age",
                Style::default().fg(health_to_color(tip_health)),
            )),
            Cell::from(Span::styled(
                format_tip_age(node.tip_age_secs()),
                Style::default().fg(health_to_color(tip_health)),
            )),
        ]),
        Row::new(vec![
            Cell::from("Slot"),
            Cell::from(format_metric_u64(metrics.slot_num)),
        ]),
        Row::new(vec![
            Cell::from("Epoch"),
            Cell::from(format_epoch_slot(metrics.epoch, metrics.slot_in_epoch)),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "Sync Progress",
                Style::default().fg(health_to_color(sync_health)),
            )),
            Cell::from(Span::styled(
                format_sync_progress(metrics.sync_progress),
                Style::default().fg(health_to_color(sync_health)),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "Connected Peers",
                Style::default().fg(health_to_color(peer_health)),
            )),
            Cell::from(Span::styled(
                format_metric_u64(metrics.peers_connected),
                Style::default().fg(health_to_color(peer_health)),
            )),
        ]),
    ];

    // Add KES row only if KES metrics are available (block producer)
    if metrics.kes_remaining.is_some() {
        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                "KES Remaining",
                Style::default().fg(health_to_color(kes_health)),
            )),
            Cell::from(Span::styled(
                format_kes_remaining(metrics.kes_remaining),
                Style::default().fg(health_to_color(kes_health)),
            )),
        ]));
    }

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Chain Status "),
    );

    frame.render_widget(table, area);
}

/// Draw epoch progress gauge
fn draw_epoch_progress(frame: &mut Frame, area: Rect, app: &App) {
    let node = app.current_node();
    let progress = node.epoch_progress().unwrap_or(0.0);
    let time_remaining = node.epoch_time_remaining();

    let label = match time_remaining {
        Some(secs) => format!(
            "{:.1}% — {} remaining",
            progress,
            format_time_remaining(secs)
        ),
        None => format!("{:.1}%", progress),
    };

    // Color based on how close to epoch end
    let gauge_color = match progress {
        p if p >= 95.0 => Color::Yellow,
        p if p >= 80.0 => Color::Cyan,
        _ => Color::Green,
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Epoch Progress "),
        )
        .gauge_style(Style::default().fg(gauge_color).bg(Color::DarkGray))
        .ratio(progress / 100.0)
        .label(Span::styled(
            label,
            Style::default().fg(Color::White).bold(),
        ));

    frame.render_widget(gauge, area);
}

/// Draw block height sparkline
fn draw_block_sparkline(frame: &mut Frame, area: Rect, app: &App) {
    let node = app.current_node();
    let data = node.history.block_height.as_slice();

    // Normalize data for display (show relative changes)
    let normalized: Vec<u64> = if let (Some(min), Some(max)) = (
        node.history.block_height.min(),
        node.history.block_height.max(),
    ) {
        let range = (max - min).max(1.0);
        data.iter()
            .map(|v| ((*v as f64 - min) / range * 100.0) as u64)
            .collect()
    } else {
        data
    };

    let title = if let Some(bpm) = node.blocks_per_minute() {
        format!(" Blocks ({:.1}/min) ", bpm)
    } else {
        " Blocks ".to_string()
    };

    let sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .data(&normalized)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(sparkline, area);
}

/// Draw resource/system metrics panel
fn draw_resource_panel(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Resource metrics table
            Constraint::Min(4),    // Memory sparkline
        ])
        .split(area);

    draw_resource_metrics(frame, chunks[0], app);
    draw_memory_sparkline(frame, chunks[1], app);
}

/// Draw resource/system metrics
fn draw_resource_metrics(frame: &mut Frame, area: Rect, app: &App) {
    let node = app.current_node();
    let metrics = &node.metrics;
    let memory_health = node.memory_health();

    let rows = vec![
        Row::new(vec![
            Cell::from("Uptime"),
            Cell::from(format_uptime(metrics.uptime_seconds)),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "Memory Used",
                Style::default().fg(health_to_color(memory_health)),
            )),
            Cell::from(Span::styled(
                format_bytes(metrics.memory_used),
                Style::default().fg(health_to_color(memory_health)),
            )),
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
    .block(Block::default().borders(Borders::ALL).title(" Resources "));

    frame.render_widget(table, area);
}

/// Draw memory usage sparkline
fn draw_memory_sparkline(frame: &mut Frame, area: Rect, app: &App) {
    let node = app.current_node();
    let data = node.history.memory_used.as_slice();

    // Normalize to show relative changes
    let normalized: Vec<u64> = if let (Some(min), Some(max)) = (
        node.history.memory_used.min(),
        node.history.memory_used.max(),
    ) {
        let range = (max - min).max(1.0);
        data.iter()
            .map(|v| ((*v as f64 - min) / range * 100.0) as u64)
            .collect()
    } else {
        data
    };

    let sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" Memory "))
        .data(&normalized)
        .style(Style::default().fg(health_to_color(node.memory_health())));

    frame.render_widget(sparkline, area);
}

/// Draw the footer with help and status
fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let node = app.current_node();
    let endpoint = format!("{}:{}", node.config.prom_host, node.config.prom_port);

    let footer_text = if let Some(ref error) = node.last_error {
        Line::from(vec![
            Span::styled(" Error: ", Style::default().fg(Color::Red).bold()),
            Span::styled(truncate_string(error, 60), Style::default().fg(Color::Red)),
        ])
    } else {
        let mut spans = vec![
            Span::styled(" [q] ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit  "),
            Span::styled("[r] ", Style::default().fg(Color::Yellow)),
            Span::raw("Refresh  "),
            Span::styled("[?] ", Style::default().fg(Color::Yellow)),
            Span::raw("Help  "),
        ];

        // Add node switching hints if multi-node
        if app.is_multi_node() {
            spans.push(Span::styled("[Tab] ", Style::default().fg(Color::Yellow)));
            spans.push(Span::raw("Next  "));
            spans.push(Span::styled("[1-9] ", Style::default().fg(Color::Yellow)));
            spans.push(Span::raw("Select  "));
        }

        spans.push(Span::raw("│ "));
        spans.push(Span::styled(endpoint, Style::default().fg(Color::DarkGray)));

        Line::from(spans)
    };

    let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

/// Draw the help popup overlay
fn draw_help_popup(frame: &mut Frame, area: Rect, is_multi_node: bool) {
    let popup_area = centered_rect(60, if is_multi_node { 60 } else { 50 }, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let mut help_lines = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().bold().underlined(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q, Esc    ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit sview"),
        ]),
        Line::from(vec![
            Span::styled("  r         ", Style::default().fg(Color::Yellow)),
            Span::raw("Force refresh metrics"),
        ]),
        Line::from(vec![
            Span::styled("  ?         ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help"),
        ]),
    ];

    // Add multi-node shortcuts if applicable
    if is_multi_node {
        help_lines.push(Line::from(vec![
            Span::styled("  Tab       ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch to next node"),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled("  Shift+Tab ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch to previous node"),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled("  1-9       ", Style::default().fg(Color::Yellow)),
            Span::raw("Select node by number"),
        ]));
    }

    help_lines.extend(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Health Indicators",
            Style::default().bold().underlined(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ● Green   ", Style::default().fg(Color::Green)),
            Span::raw("Healthy"),
        ]),
        Line::from(vec![
            Span::styled("  ● Yellow  ", Style::default().fg(Color::Yellow)),
            Span::raw("Warning (needs attention)"),
        ]),
        Line::from(vec![
            Span::styled("  ● Red     ", Style::default().fg(Color::Red)),
            Span::raw("Critical (action required)"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray).italic(),
        )),
    ]);

    let help = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(help, popup_area);
}

/// Create a centered rectangle for popups
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// ============================================================================
// Formatting helpers
// ============================================================================

fn health_to_color(status: HealthStatus) -> Color {
    match status {
        HealthStatus::Good => Color::Green,
        HealthStatus::Warning => Color::Yellow,
        HealthStatus::Critical => Color::Red,
    }
}

fn format_metric_u64(value: Option<u64>) -> String {
    value.map(format_number).unwrap_or_else(|| "—".to_string())
}

fn format_number(n: u64) -> String {
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
        Some(s) if s >= 86400.0 => {
            let days = s / 86400.0;
            format!("{:.1}d", days)
        }
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
        Some(p) if p >= 99.9 => "100% ✓".to_string(),
        Some(p) => format!("{:.2}%", p),
        None => "—".to_string(),
    }
}

fn format_uptime(seconds: Option<f64>) -> String {
    match seconds {
        Some(s) => {
            let total_secs = s as u64;
            let days = total_secs / 86400;
            let hours = (total_secs % 86400) / 3600;
            let mins = (total_secs % 3600) / 60;

            if days > 0 {
                format!("{}d {}h {}m", days, hours, mins)
            } else if hours > 0 {
                format!("{}h {}m", hours, mins)
            } else {
                format!("{}m", mins)
            }
        }
        None => "—".to_string(),
    }
}

fn format_kes_remaining(periods: Option<u64>) -> String {
    match periods {
        Some(p) => {
            let days_approx = (p as f64 * 1.5) as u64;
            format!("{} (~{}d)", p, days_approx)
        }
        None => "—".to_string(),
    }
}

fn format_tip_age(seconds: Option<u64>) -> String {
    match seconds {
        Some(s) if s < 60 => format!("{}s ago", s),
        Some(s) if s < 3600 => format!("{}m {}s ago", s / 60, s % 60),
        Some(s) => format!("{}h {}m ago", s / 3600, (s % 3600) / 60),
        None => "—".to_string(),
    }
}

fn format_epoch_slot(epoch: Option<u64>, slot_in_epoch: Option<u64>) -> String {
    match (epoch, slot_in_epoch) {
        (Some(e), Some(s)) => format!("{} (slot {})", format_number(e), format_number(s)),
        (Some(e), None) => format_number(e),
        _ => "—".to_string(),
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn format_time_remaining(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}
