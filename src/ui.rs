//! User interface rendering
//!
//! This module handles all TUI rendering using ratatui.

use crate::app::{App, AppMode, HealthStatus};
use crate::themes::Palette;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Sparkline, Table, Tabs, Wrap},
};

/// Main draw function - renders the entire UI
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let palette = app.theme.palette();

    // Create main layout - add node tabs if multi-node mode
    let chunks = if app.is_multi_node() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Node tabs
                Constraint::Length(3), // Header
                Constraint::Min(15),   // Main content (increased for more metrics)
                Constraint::Length(3), // Footer/status
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(15),   // Main content (increased for more metrics)
                Constraint::Length(3), // Footer/status
            ])
            .split(area)
    };

    // Draw node tabs if multi-node mode
    let (header_area, main_area, footer_area) = if app.is_multi_node() {
        draw_node_tabs(frame, chunks[0], app, &palette);
        (chunks[1], chunks[2], chunks[3])
    } else {
        (chunks[0], chunks[1], chunks[2])
    };

    // Draw header
    draw_header(frame, header_area, app, &palette);

    // Draw main content - with improved 3-column layout
    draw_main(frame, main_area, app, &palette);

    // Draw footer
    draw_footer(frame, footer_area, app, &palette);

    // Draw help overlay if in help mode
    if app.mode == AppMode::Help {
        draw_help_popup(frame, area, app.is_multi_node(), &palette);
    }
}

/// Draw the node selection tabs
fn draw_node_tabs(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let titles: Vec<Line> = app
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let health_color = health_to_color(node.overall_health(), palette);
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
                    Style::default().fg(palette.text_muted),
                ),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Nodes ").border_style(Style::default().fg(palette.border)))
        .select(app.selected_node)
        .style(Style::default().fg(palette.text))
        .highlight_style(
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" │ ");

    frame.render_widget(tabs, area);
}

/// Draw the header section with node name and status
fn draw_header(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let health_color = health_to_color(node.overall_health(), palette);
    let status_indicator = if node.metrics.connected { "●" } else { "○" };

    let role_badge = match node.role {
        crate::config::NodeRole::Bp => {
            Span::styled(" [BP] ", Style::default().fg(palette.secondary).bold())
        }
        crate::config::NodeRole::Relay => Span::raw(""),
    };

    let header_text = Line::from(vec![
        Span::styled(
            format!(" {} ", node.config.node_name),
            Style::default().bold().fg(palette.text),
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
            Style::default().fg(palette.primary),
        ),
        Span::raw(" │ "),
        Span::styled(
            format!("Node: {}", node.metrics.node_type),
            Style::default().fg(palette.tertiary),
        ),
        Span::raw(" │ "),
        Span::styled(
            format!(" [{}] ", app.theme.display_name()),
            Style::default().fg(palette.text_muted),
        ),
    ]);

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" sview ")
            .border_style(Style::default().fg(palette.border)),
    );

    frame.render_widget(header, area);
}

/// Draw the main content area with metrics
fn draw_main(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    // Split into: epoch gauge, metrics, and sparklines
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Epoch progress gauge (full width)
            Constraint::Min(15),    // Metrics section (3 columns)
            Constraint::Length(5),  // Sparklines section (side-by-side)
        ])
        .split(area);

    // Epoch progress gauge (full width)
    draw_epoch_progress(frame, chunks[0], app, palette);

    // Metrics section: 3-column layout
    draw_metrics_section(frame, chunks[1], app, palette);

    // Sparklines section: Side-by-side
    draw_sparklines_section(frame, chunks[2], app, palette);
}

/// Draw the metrics section (chain, network, resources in 3 columns)
fn draw_metrics_section(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)])
        .split(area);

    // Left column: Chain Metrics
    draw_chain_metrics_compact(frame, chunks[0], app, palette);

    // Middle column: Network & Peer Metrics
    draw_network_panel(frame, chunks[1], app, palette);

    // Right column: Resource & System Metrics
    draw_resource_metrics_compact(frame, chunks[2], app, palette);
}

/// Draw the sparklines section (block + memory side-by-side)
fn draw_sparklines_section(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_block_sparkline(frame, chunks[0], app, palette);
    draw_memory_sparkline(frame, chunks[1], app, palette);
}

/// Draw compact chain metrics (epoch gauge moved to full-width section)
fn draw_chain_metrics_compact(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    draw_chain_metrics(frame, area, app, palette);
}

/// Draw chain metrics table
fn draw_chain_metrics(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
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
                Style::default().fg(health_to_color(tip_health, palette)),
            )),
            Cell::from(Span::styled(
                format_tip_age(node.tip_age_secs()),
                Style::default().fg(health_to_color(tip_health, palette)),
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
                Style::default().fg(health_to_color(sync_health, palette)),
            )),
            Cell::from(Span::styled(
                format_sync_progress(metrics.sync_progress),
                Style::default().fg(health_to_color(sync_health, palette)),
            )),
        ]),
        Row::new(vec![
            Cell::from("Chain Density"),
            Cell::from(format_density(metrics.density)),
        ]),
        Row::new(vec![
            Cell::from("TX Processed"),
            Cell::from(format_metric_u64(metrics.tx_processed)),
        ]),
        Row::new(vec![
            Cell::from("Forks"),
            Cell::from(format_metric_u64(metrics.forks)),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "Connected Peers",
                Style::default().fg(health_to_color(peer_health, palette)),
            )),
            Cell::from(Span::styled(
                format_metric_u64(metrics.peers_connected),
                Style::default().fg(health_to_color(peer_health, palette)),
            )),
        ]),
    ];

    // Add KES row only if KES metrics are available (block producer)
    if metrics.kes_remaining.is_some() {
        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                "KES Remaining",
                Style::default().fg(health_to_color(kes_health, palette)),
            )),
            Cell::from(Span::styled(
                format_kes_remaining(metrics.kes_remaining),
                Style::default().fg(health_to_color(kes_health, palette)),
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
            .title(" Chain Status ")
            .border_style(Style::default().fg(palette.border)),
    );

    frame.render_widget(table, area);
}

/// Draw epoch progress gauge
fn draw_epoch_progress(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
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
        p if p >= 95.0 => palette.warning,
        p if p >= 80.0 => palette.primary,
        _ => palette.healthy,
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Epoch Progress ")
                .border_style(Style::default().fg(palette.border)),
        )
        .gauge_style(Style::default().fg(gauge_color).bg(Color::DarkGray))
        .ratio(progress / 100.0)
        .label(Span::styled(
            label,
            Style::default().fg(palette.text).bold(),
        ));

    frame.render_widget(gauge, area);
}

/// Draw block height sparkline
fn draw_block_sparkline(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(palette.border)),
        )
        .data(&normalized)
        .style(Style::default().fg(palette.sparkline));

    frame.render_widget(sparkline, area);
}

/// Draw network and peer metrics panel
fn draw_network_panel(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Connection metrics
            Constraint::Min(4),     // P2P peer breakdown
        ])
        .split(area);

    draw_connection_metrics(frame, chunks[0], app, palette);
    draw_peer_breakdown(frame, chunks[1], app, palette);
}

/// Draw connection and block fetch metrics
fn draw_connection_metrics(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let metrics = &node.metrics;

    let rows = vec![
        Row::new(vec![
            Cell::from("Incoming"),
            Cell::from(format_metric_u64(metrics.incoming_connections)),
        ]),
        Row::new(vec![
            Cell::from("Outgoing"),
            Cell::from(format_metric_u64(metrics.outgoing_connections)),
        ]),
        Row::new(vec![
            Cell::from("Duplex"),
            Cell::from(format_metric_u64(metrics.full_duplex_connections)),
        ]),
        Row::new(vec![
            Cell::from("Unidirectional"),
            Cell::from(format_metric_u64(metrics.unidirectional_connections)),
        ]),
        Row::new(vec![
            Cell::from("Prunable"),
            Cell::from(format_metric_u64(metrics.prunable_connections)),
        ]),
        Row::new(vec![
            Cell::from("Block Delay"),
            Cell::from(format_block_delay(metrics.block_delay_s)),
        ]),
        Row::new(vec![
            Cell::from("Blocks Served"),
            Cell::from(format_metric_u64(metrics.blocks_served)),
        ]),
        Row::new(vec![
            Cell::from("Blocks Late"),
            Cell::from(format_metric_u64(metrics.blocks_late)),
        ]),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Network & Block Fetch ")
            .border_style(Style::default().fg(palette.border)),
    );

    frame.render_widget(table, area);
}

/// Draw P2P peer classification breakdown
fn draw_peer_breakdown(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let metrics = &node.metrics;
    let p2p = &metrics.p2p;

    let rows = vec![
        Row::new(vec![
            Cell::from("Cold Peers"),
            Cell::from(format_metric_u64(p2p.cold_peers)),
        ]),
        Row::new(vec![
            Cell::from("Warm Peers"),
            Cell::from(format_metric_u64(p2p.warm_peers)),
        ]),
        Row::new(vec![
            Cell::from("Hot Peers"),
            Cell::from(format_metric_u64(p2p.hot_peers)),
        ]),
        Row::new(vec![
            Cell::from("Duplex Peers"),
            Cell::from(format_metric_u64(p2p.duplex_peers)),
        ]),
        Row::new(vec![
            Cell::from("Bidirectional"),
            Cell::from(format_metric_u64(p2p.bidirectional_peers)),
        ]),
        Row::new(vec![
            Cell::from("Unidirectional"),
            Cell::from(format_metric_u64(p2p.unidirectional_peers)),
        ]),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" P2P Peer Classification ")
            .border_style(Style::default().fg(palette.border)),
    );

    frame.render_widget(table, area);
}

/// Draw compact resource metrics (without sparkline)
fn draw_resource_metrics_compact(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    draw_resource_metrics(frame, area, app, palette);
}

/// Draw resource/system metrics
fn draw_resource_metrics(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let metrics = &node.metrics;
    let memory_health = node.memory_health();

    let mut rows = vec![
        Row::new(vec![
            Cell::from("Uptime"),
            Cell::from(format_uptime(metrics.uptime_seconds)),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "Memory Used",
                Style::default().fg(health_to_color(memory_health, palette)),
            )),
            Cell::from(Span::styled(
                format_bytes(metrics.memory_used),
                Style::default().fg(health_to_color(memory_health, palette)),
            )),
        ]),
        Row::new(vec![
            Cell::from("Memory Heap"),
            Cell::from(format_bytes(metrics.memory_heap)),
        ]),
        Row::new(vec![
            Cell::from("GC Minor"),
            Cell::from(format_metric_u64(metrics.gc_minor)),
        ]),
        Row::new(vec![
            Cell::from("GC Major"),
            Cell::from(format_metric_u64(metrics.gc_major)),
        ]),
        Row::new(vec![
            Cell::from("CPU Time"),
            Cell::from(format_cpu_ms(metrics.cpu_ms)),
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

    // Add forging metrics if available (block producer)
    if metrics.blocks_adopted.is_some() || metrics.blocks_didnt_adopt.is_some() {
        rows.push(Row::new(vec![
            Cell::from("Blocks Adopted"),
            Cell::from(format_metric_u64(metrics.blocks_adopted)),
        ]));
        rows.push(Row::new(vec![
            Cell::from("Blocks Failed"),
            Cell::from(format_metric_u64(metrics.blocks_didnt_adopt)),
        ]));
    }

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Resources & Forging ")
            .border_style(Style::default().fg(palette.border)),
    );

    frame.render_widget(table, area);
}

/// Draw memory usage sparkline
fn draw_memory_sparkline(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Memory ")
                .border_style(Style::default().fg(palette.border)),
        )
        .data(&normalized)
        .style(Style::default().fg(health_to_color(node.memory_health(), palette)));

    frame.render_widget(sparkline, area);
}

/// Draw the footer with help and status
fn draw_footer(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let endpoint = format!("{}:{}", node.config.prom_host, node.config.prom_port);

    let footer_text = if let Some(ref error) = node.last_error {
        Line::from(vec![
            Span::styled(" Error: ", Style::default().fg(palette.critical).bold()),
            Span::styled(truncate_string(error, 60), Style::default().fg(palette.critical)),
        ])
    } else {
        let mut spans = vec![
            Span::styled(" [q] ", Style::default().fg(palette.tertiary)),
            Span::raw("Quit  "),
            Span::styled("[r] ", Style::default().fg(palette.tertiary)),
            Span::raw("Refresh  "),
            Span::styled("[?] ", Style::default().fg(palette.tertiary)),
            Span::raw("Help  "),
            Span::styled("[t] ", Style::default().fg(palette.tertiary)),
            Span::raw("Theme  "),
        ];

        // Add node switching hints if multi-node
        if app.is_multi_node() {
            spans.push(Span::styled("[Tab] ", Style::default().fg(palette.tertiary)));
            spans.push(Span::raw("Next  "));
            spans.push(Span::styled("[1-9] ", Style::default().fg(palette.tertiary)));
            spans.push(Span::raw("Select  "));
        }

        spans.push(Span::raw("│ "));
        spans.push(Span::styled(endpoint, Style::default().fg(palette.text_muted)));

        Line::from(spans)
    };

    let footer = Paragraph::new(footer_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)),
    );

    frame.render_widget(footer, area);
}

/// Draw the help popup overlay
fn draw_help_popup(frame: &mut Frame, area: Rect, is_multi_node: bool, palette: &Palette) {
    let popup_area = centered_rect(60, if is_multi_node { 60 } else { 50 }, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let mut help_lines = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().bold().underlined().fg(palette.primary),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q, Esc    ", Style::default().fg(palette.tertiary)),
            Span::raw("Quit sview"),
        ]),
        Line::from(vec![
            Span::styled("  r         ", Style::default().fg(palette.tertiary)),
            Span::raw("Force refresh metrics"),
        ]),
        Line::from(vec![
            Span::styled("  t         ", Style::default().fg(palette.tertiary)),
            Span::raw("Cycle color theme"),
        ]),
        Line::from(vec![
            Span::styled("  ?         ", Style::default().fg(palette.tertiary)),
            Span::raw("Toggle this help"),
        ]),
    ];

    // Add multi-node shortcuts if applicable
    if is_multi_node {
        help_lines.push(Line::from(""));
        help_lines.push(Line::from(Span::styled(
            "Multi-Node Navigation",
            Style::default().bold().underlined().fg(palette.primary),
        )));
        help_lines.push(Line::from(""));
        help_lines.push(Line::from(vec![
            Span::styled("  Tab       ", Style::default().fg(palette.tertiary)),
            Span::raw("Next node"),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled("  Shift+Tab ", Style::default().fg(palette.tertiary)),
            Span::raw("Previous node"),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled("  ← →       ", Style::default().fg(palette.tertiary)),
            Span::raw("Switch nodes"),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled("  1-9       ", Style::default().fg(palette.tertiary)),
            Span::raw("Select node by number"),
        ]));
    }

    help_lines.extend(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Health Indicators",
            Style::default().bold().underlined().fg(palette.primary),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ● Healthy   ", Style::default().fg(palette.healthy)),
            Span::raw("Good status"),
        ]),
        Line::from(vec![
            Span::styled("  ● Warning   ", Style::default().fg(palette.warning)),
            Span::raw("Needs attention"),
        ]),
        Line::from(vec![
            Span::styled("  ● Critical  ", Style::default().fg(palette.critical)),
            Span::raw("Action required"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(palette.text_muted).italic(),
        )),
    ]);

    let help = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(palette.primary)),
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

fn health_to_color(status: HealthStatus, palette: &Palette) -> Color {
    match status {
        HealthStatus::Good => palette.healthy,
        HealthStatus::Warning => palette.warning,
        HealthStatus::Critical => palette.critical,
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

#[allow(dead_code)]
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

fn format_cpu_ms(ms: Option<u64>) -> String {
    match ms {
        Some(total_ms) => {
            let secs = total_ms / 1000;
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let sec = secs % 60;

            if hours > 0 {
                format!("{}h {}m {}s", hours, mins, sec)
            } else if mins > 0 {
                format!("{}m {}s", mins, sec)
            } else {
                format!("{}s", sec)
            }
        }
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

fn format_density(density: Option<f64>) -> String {
    match density {
        Some(d) => format!("{:.4}", d),
        None => "—".to_string(),
    }
}

fn format_block_delay(secs: Option<f64>) -> String {
    match secs {
        Some(s) if s < 0.001 => "< 1ms".to_string(),
        Some(s) if s < 1.0 => format!("{:.1}ms", s * 1000.0),
        Some(s) => format!("{:.2}s", s),
        None => "—".to_string(),
    }
}
