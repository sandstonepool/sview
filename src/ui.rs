//! User interface rendering
//!
//! This module handles all TUI rendering using ratatui.

use crate::app::{App, AppMode, HealthStatus};
use crate::themes::Palette;
use ratatui::{
    prelude::*,
    symbols,
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
                Constraint::Length(3), // Header with status indicators
                Constraint::Min(10),   // Main content
                Constraint::Length(2), // Footer/status
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with status indicators
                Constraint::Min(10),   // Main content
                Constraint::Length(2), // Footer/status
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

    // Draw header with health indicators
    draw_header(frame, header_area, app, &palette);

    // Draw main content - 2-row layout with sparklines
    draw_main_content(frame, main_area, app, &palette);

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
                crate::config::NodeRole::Bp => " BP",
                crate::config::NodeRole::Relay => "",
            };
            Line::from(vec![
                Span::styled(indicator, Style::default().fg(health_color)),
                Span::raw(" "),
                Span::styled(
                    format!("{}{}", node.config.node_name, role_suffix),
                    Style::default().fg(palette.text),
                ),
                Span::styled(
                    format!(" [{}]", i + 1),
                    Style::default().fg(palette.text_muted),
                ),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Nodes ")
                .border_style(Style::default().fg(palette.border)),
        )
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

/// Draw the header section with health indicators
fn draw_header(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let metrics = &node.metrics;

    // Build status line with key health indicators
    let sync_health = node.sync_health();
    let peer_health = node.peer_health();
    let tip_health = node.tip_health();
    let mem_health = node.memory_health();

    let status_indicator = if node.metrics.connected {
        Span::styled("● ONLINE", Style::default().fg(palette.healthy).bold())
    } else {
        Span::styled("○ OFFLINE", Style::default().fg(palette.critical).bold())
    };

    let role_badge = match node.role {
        crate::config::NodeRole::Bp => Span::styled(
            " [BLOCK PRODUCER] ",
            Style::default().fg(palette.secondary).bold(),
        ),
        crate::config::NodeRole::Relay => {
            Span::styled(" [RELAY] ", Style::default().fg(palette.tertiary))
        }
    };

    // Quick health indicators
    let sync_dot = Span::styled(
        "●",
        Style::default().fg(health_to_color(sync_health, palette)),
    );
    let peer_dot = Span::styled(
        "●",
        Style::default().fg(health_to_color(peer_health, palette)),
    );
    let tip_dot = Span::styled(
        "●",
        Style::default().fg(health_to_color(tip_health, palette)),
    );
    let mem_dot = Span::styled(
        "●",
        Style::default().fg(health_to_color(mem_health, palette)),
    );

    // Format key metrics for header
    let block_str = metrics
        .block_height
        .map(format_number)
        .unwrap_or_else(|| "—".to_string());
    let epoch_str = metrics
        .epoch
        .map(|e| format!("E{}", e))
        .unwrap_or_else(|| "—".to_string());
    let peers_str = metrics
        .peers_connected
        .map(|p| p.to_string())
        .unwrap_or_else(|| "—".to_string());

    let header_text = Line::from(vec![
        Span::styled(
            format!(" {} ", node.config.node_name),
            Style::default().bold().fg(palette.primary),
        ),
        role_badge,
        status_indicator,
        Span::raw("  │  "),
        Span::styled("Block: ", Style::default().fg(palette.text_muted)),
        Span::styled(block_str, Style::default().fg(palette.text)),
        Span::raw("  "),
        Span::styled(epoch_str, Style::default().fg(palette.tertiary)),
        Span::raw("  │  "),
        Span::styled("Peers: ", Style::default().fg(palette.text_muted)),
        Span::styled(peers_str, Style::default().fg(palette.text)),
        Span::raw("  │  "),
        Span::styled("Health: ", Style::default().fg(palette.text_muted)),
        sync_dot,
        Span::styled("Sync ", Style::default().fg(palette.text_muted)),
        peer_dot,
        Span::styled("Peers ", Style::default().fg(palette.text_muted)),
        tip_dot,
        Span::styled("Tip ", Style::default().fg(palette.text_muted)),
        mem_dot,
        Span::styled("Mem", Style::default().fg(palette.text_muted)),
    ]);

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" sview — {} ", node.config.network))
            .border_style(Style::default().fg(palette.border)),
    );

    frame.render_widget(header, area);
}

/// Draw the main content area
fn draw_main_content(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    // Two-row layout: gauges on top, metrics + sparklines below
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Progress gauges
            Constraint::Min(8),    // Metrics and sparklines
        ])
        .split(area);

    // Top row: Epoch progress + Sync + Memory gauges
    draw_gauge_row(frame, chunks[0], app, palette);

    // Bottom row: Metrics tables + Sparklines
    draw_metrics_and_sparklines(frame, chunks[1], app, palette);
}

/// Draw the gauge row (epoch, sync, memory)
fn draw_gauge_row(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Epoch progress
            Constraint::Percentage(25), // Sync progress
            Constraint::Percentage(25), // Memory usage
        ])
        .split(area);

    draw_epoch_gauge(frame, chunks[0], app, palette);
    draw_sync_gauge(frame, chunks[1], app, palette);
    draw_memory_gauge(frame, chunks[2], app, palette);
}

/// Draw epoch progress gauge
fn draw_epoch_gauge(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let progress = node.epoch_progress().unwrap_or(0.0);
    let time_remaining = node.epoch_time_remaining();

    let label = match (node.metrics.epoch, time_remaining) {
        (Some(epoch), Some(secs)) => format!(
            "Epoch {} — {:.1}% — {} left",
            epoch,
            progress,
            format_time_remaining(secs)
        ),
        (Some(epoch), None) => format!("Epoch {} — {:.1}%", epoch, progress),
        _ => format!("{:.1}%", progress),
    };

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

/// Draw sync progress gauge
fn draw_sync_gauge(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let progress = node.metrics.sync_progress.unwrap_or(0.0);
    let sync_health = node.sync_health();

    let label = if progress >= 99.9 {
        "Synced ✓".to_string()
    } else {
        format!("{:.2}%", progress)
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Sync ")
                .border_style(Style::default().fg(palette.border)),
        )
        .gauge_style(
            Style::default()
                .fg(health_to_color(sync_health, palette))
                .bg(Color::DarkGray),
        )
        .ratio((progress / 100.0).min(1.0))
        .label(Span::styled(
            label,
            Style::default().fg(palette.text).bold(),
        ));

    frame.render_widget(gauge, area);
}

/// Draw memory usage gauge
fn draw_memory_gauge(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let metrics = &node.metrics;
    let memory_health = node.memory_health();

    let label = match metrics.memory_used {
        Some(bytes) => format_bytes(Some(bytes)),
        None => "—".to_string(),
    };

    let ratio = if let (Some(used), Some(heap)) = (metrics.memory_used, metrics.memory_heap) {
        (used as f64 / heap as f64).min(1.0)
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Memory ")
                .border_style(Style::default().fg(palette.border)),
        )
        .gauge_style(
            Style::default()
                .fg(health_to_color(memory_health, palette))
                .bg(Color::DarkGray),
        )
        .ratio(ratio)
        .label(Span::styled(
            label,
            Style::default().fg(palette.text).bold(),
        ));

    frame.render_widget(gauge, area);
}

/// Draw metrics tables and sparklines
fn draw_metrics_and_sparklines(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    // 3-column layout: Chain | Network | Resources, with sparklines on right
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Chain metrics
            Constraint::Percentage(30), // Network metrics
            Constraint::Percentage(40), // Resources + Sparklines
        ])
        .split(area);

    draw_chain_metrics(frame, columns[0], app, palette);
    draw_network_metrics(frame, columns[1], app, palette);
    draw_resources_with_sparklines(frame, columns[2], app, palette);
}

/// Draw chain metrics table
fn draw_chain_metrics(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let metrics = &node.metrics;
    let tip_health = node.tip_health();
    let kes_health = node.kes_health();

    let mut rows = vec![
        create_metric_row_with_trend(
            "Block Height",
            format_metric_u64(metrics.block_height),
            node.history.block_height.trend(),
            palette,
        ),
        create_health_row(
            "Tip Age",
            format_tip_age(node.tip_age_secs()),
            tip_health,
            palette,
        ),
        create_metric_row("Slot", format_metric_u64(metrics.slot_num), palette),
        create_metric_row(
            "Slot in Epoch",
            format_metric_u64(metrics.slot_in_epoch),
            palette,
        ),
        create_metric_row("Density", format_density(metrics.density), palette),
        create_metric_row(
            "TX Processed",
            format_metric_u64(metrics.tx_processed),
            palette,
        ),
        create_metric_row("Forks", format_metric_u64(metrics.forks), palette),
    ];

    // Add KES row only if available (block producer)
    if metrics.kes_remaining.is_some() {
        rows.push(create_health_row(
            "KES Remaining",
            format_kes_remaining(metrics.kes_remaining),
            kes_health,
            palette,
        ));
    }

    // Add OpCert validation if available (block producer)
    if let (Some(disk), Some(chain)) = (metrics.op_cert_counter_disk, metrics.op_cert_counter_chain)
    {
        let (op_cert_status, op_cert_health) = if disk == chain {
            (format!("✓ {} (valid)", disk), HealthStatus::Good)
        } else if disk > chain {
            (
                format!("⚠ disk:{} chain:{}", disk, chain),
                HealthStatus::Warning,
            )
        } else {
            (
                format!("✗ disk:{} < chain:{}", disk, chain),
                HealthStatus::Critical,
            )
        };
        rows.push(create_health_row(
            "OpCert",
            op_cert_status,
            op_cert_health,
            palette,
        ));
    }

    // Add forging metrics if available
    if metrics.forging_enabled.is_some() {
        let forging_str = if metrics.forging_enabled.unwrap_or(false) {
            "Enabled"
        } else {
            "Disabled"
        };
        rows.push(create_metric_row(
            "Forging",
            forging_str.to_string(),
            palette,
        ));
    }

    if metrics.blocks_adopted.is_some() {
        rows.push(create_metric_row(
            "Blocks Forged",
            format_metric_u64(metrics.blocks_adopted),
            palette,
        ));
    }

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Chain ")
            .border_style(Style::default().fg(palette.border)),
    );

    frame.render_widget(table, area);
}

/// Draw network and peer metrics
fn draw_network_metrics(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let metrics = &node.metrics;
    let peer_health = node.peer_health();
    let peer_trend = node.history.peers_connected.trend();

    // Format connected peers with trend indicator
    let connected_value = format_metric_u64(metrics.peers_connected);
    let (trend_indicator, _) = format_trend(peer_trend, palette);
    let connected_with_trend = if !trend_indicator.is_empty() {
        format!("{} {}", connected_value, trend_indicator)
    } else {
        connected_value
    };

    let rows = vec![
        create_health_row("Connected", connected_with_trend, peer_health, palette),
        create_metric_row(
            "Incoming",
            format_metric_u64(metrics.incoming_connections),
            palette,
        ),
        create_metric_row(
            "Outgoing",
            format_metric_u64(metrics.outgoing_connections),
            palette,
        ),
        create_metric_row(
            "Duplex",
            format_metric_u64(metrics.full_duplex_connections),
            palette,
        ),
        create_metric_row(
            "Unidirectional",
            format_metric_u64(metrics.unidirectional_connections),
            palette,
        ),
        create_separator_row(palette),
        // Peer distribution bar showing hot/warm/cold ratio
        create_metric_row(
            "Peer Dist",
            format_peer_distribution(
                metrics.p2p.hot_peers,
                metrics.p2p.warm_peers,
                metrics.p2p.cold_peers,
            ),
            palette,
        ),
        create_separator_row(palette),
        create_metric_row(
            "Block Delay",
            format_block_delay(metrics.block_delay_s),
            palette,
        ),
        create_metric_row(
            "Blocks Served",
            format_metric_u64(metrics.blocks_served),
            palette,
        ),
        create_separator_row(palette),
        // Block propagation CDF (percentage of blocks received within time threshold)
        create_metric_row(
            "Prop ≤1s",
            format_cdf_percent(metrics.block_delay_cdf_1s),
            palette,
        ),
        create_metric_row(
            "Prop ≤3s",
            format_cdf_percent(metrics.block_delay_cdf_3s),
            palette,
        ),
        create_metric_row(
            "Prop ≤5s",
            format_cdf_percent(metrics.block_delay_cdf_5s),
            palette,
        ),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(55), Constraint::Percentage(45)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Network & Peers ")
            .border_style(Style::default().fg(palette.border)),
    );

    frame.render_widget(table, area);
}

/// Draw resources with sparklines
fn draw_resources_with_sparklines(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Resource metrics
            Constraint::Min(4),    // Sparklines
        ])
        .split(area);

    draw_resource_metrics(frame, chunks[0], app, palette);
    draw_sparklines(frame, chunks[1], app, palette);
}

/// Draw resource metrics table
fn draw_resource_metrics(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let metrics = &node.metrics;
    let memory_health = node.memory_health();

    let rows = vec![
        create_metric_row("Uptime", format_uptime(metrics.uptime_seconds), palette),
        create_health_row(
            "Memory Used",
            format_bytes(metrics.memory_used),
            memory_health,
            palette,
        ),
        create_metric_row("Memory Heap", format_bytes(metrics.memory_heap), palette),
        create_metric_row("GC Minor", format_metric_u64(metrics.gc_minor), palette),
        create_metric_row("GC Major", format_metric_u64(metrics.gc_major), palette),
        create_metric_row(
            "Mempool TXs",
            format_metric_u64(metrics.mempool_txs),
            palette,
        ),
        create_metric_row("Mempool Size", format_bytes(metrics.mempool_bytes), palette),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(55), Constraint::Percentage(45)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Resources ")
            .border_style(Style::default().fg(palette.border)),
    );

    frame.render_widget(table, area);
}

/// Draw sparklines for historical data
fn draw_sparklines(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();
    let history = &node.history;

    // Split into two sparklines side by side
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Block height sparkline (show trend)
    let block_data = history.block_height.as_slice();
    if !block_data.is_empty() {
        // Normalize to show relative changes
        let min_val = block_data.iter().min().copied().unwrap_or(0);
        let normalized: Vec<u64> = block_data
            .iter()
            .map(|v| v.saturating_sub(min_val))
            .collect();

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Block Height ")
                    .border_style(Style::default().fg(palette.border)),
            )
            .data(&normalized)
            .style(Style::default().fg(palette.sparkline))
            .bar_set(symbols::bar::NINE_LEVELS);

        frame.render_widget(sparkline, chunks[0]);
    } else {
        let empty = Paragraph::new("No history").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Block Height ")
                .border_style(Style::default().fg(palette.border)),
        );
        frame.render_widget(empty, chunks[0]);
    }

    // Memory sparkline
    let mem_data = history.memory_used.as_slice();
    if !mem_data.is_empty() {
        // Normalize to fit in sparkline range
        let max_val = mem_data.iter().max().copied().unwrap_or(1);
        let scale = if max_val > 0 {
            100.0 / max_val as f64
        } else {
            1.0
        };
        let normalized: Vec<u64> = mem_data
            .iter()
            .map(|v| (*v as f64 * scale) as u64)
            .collect();

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Memory ")
                    .border_style(Style::default().fg(palette.border)),
            )
            .data(&normalized)
            .style(Style::default().fg(palette.gauge))
            .bar_set(symbols::bar::NINE_LEVELS);

        frame.render_widget(sparkline, chunks[1]);
    } else {
        let empty = Paragraph::new("No history").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Memory ")
                .border_style(Style::default().fg(palette.border)),
        );
        frame.render_widget(empty, chunks[1]);
    }
}

/// Draw the footer with help hints and last update time
fn draw_footer(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let node = app.current_node();

    // Build footer spans
    let mut spans = vec![];

    // Show error if present
    if let Some(ref error) = node.last_error {
        spans.push(Span::styled(
            format!(" ⚠ {} ", truncate_string(error, 50)),
            Style::default().fg(palette.critical),
        ));
        spans.push(Span::raw(" │ "));
    }

    // Help hints
    spans.extend(vec![
        Span::styled(" q", Style::default().fg(palette.tertiary)),
        Span::raw(" quit "),
        Span::styled("r", Style::default().fg(palette.tertiary)),
        Span::raw(" refresh "),
        Span::styled("t", Style::default().fg(palette.tertiary)),
        Span::raw(" theme "),
        Span::styled("?", Style::default().fg(palette.tertiary)),
        Span::raw(" help"),
    ]);

    // Add node switching hints if multi-node
    if app.is_multi_node() {
        spans.extend(vec![
            Span::raw(" │ "),
            Span::styled("Tab", Style::default().fg(palette.tertiary)),
            Span::raw(" next "),
            Span::styled("1-9", Style::default().fg(palette.tertiary)),
            Span::raw(" select"),
        ]);
    }

    // Last update time
    if let Some(last_fetch) = node.last_fetch_time {
        let elapsed = last_fetch.elapsed().as_secs();
        let update_str = if elapsed < 2 {
            "just now".to_string()
        } else {
            format!("{}s ago", elapsed)
        };
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(
            format!("Updated {}", update_str),
            Style::default().fg(palette.text_muted),
        ));
    }

    // Theme name
    spans.push(Span::raw(" │ "));
    spans.push(Span::styled(
        app.theme.display_name(),
        Style::default().fg(palette.text_muted),
    ));

    let footer = Paragraph::new(Line::from(spans));
    frame.render_widget(footer, area);
}

/// Draw the help popup overlay
fn draw_help_popup(frame: &mut Frame, area: Rect, is_multi_node: bool, palette: &Palette) {
    let popup_area = centered_rect(60, if is_multi_node { 65 } else { 55 }, area);

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
// Table row helpers
// ============================================================================

fn create_metric_row<'a>(label: &'a str, value: String, palette: &Palette) -> Row<'a> {
    Row::new(vec![
        Cell::from(Span::styled(label, Style::default().fg(palette.text_muted))),
        Cell::from(Span::styled(value, Style::default().fg(palette.text))),
    ])
}

fn create_metric_row_with_trend<'a>(
    label: &'a str,
    value: String,
    trend: Option<f64>,
    palette: &Palette,
) -> Row<'a> {
    let (trend_indicator, trend_color) = format_trend(trend, palette);
    let value_with_trend = if !trend_indicator.is_empty() {
        format!("{} {}", value, trend_indicator)
    } else {
        value
    };
    Row::new(vec![
        Cell::from(Span::styled(label, Style::default().fg(palette.text_muted))),
        Cell::from(Span::styled(
            value_with_trend,
            Style::default().fg(trend_color),
        )),
    ])
}

/// Format a trend value into an indicator arrow and color
fn format_trend(trend: Option<f64>, palette: &Palette) -> (&'static str, Color) {
    match trend {
        Some(t) if t > 0.5 => ("↑", palette.healthy),
        Some(t) if t < -0.5 => ("↓", palette.critical),
        Some(_) => ("→", palette.text),
        None => ("", palette.text),
    }
}

fn create_health_row<'a>(
    label: &'a str,
    value: String,
    health: HealthStatus,
    palette: &Palette,
) -> Row<'a> {
    let color = health_to_color(health, palette);
    Row::new(vec![
        Cell::from(Span::styled(label, Style::default().fg(color))),
        Cell::from(Span::styled(value, Style::default().fg(color))),
    ])
}

fn create_separator_row(palette: &Palette) -> Row<'static> {
    Row::new(vec![
        Cell::from(Span::styled(
            "─────────",
            Style::default().fg(palette.border),
        )),
        Cell::from(Span::styled(
            "────────",
            Style::default().fg(palette.border),
        )),
    ])
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
        Some(s) if s < 1.0 => format!("{:.0}ms", s * 1000.0),
        Some(s) => format!("{:.2}s", s),
        None => "—".to_string(),
    }
}

/// Format CDF (cumulative distribution function) value as percentage
/// The CDF value represents the fraction of blocks received within the threshold
fn format_cdf_percent(cdf: Option<f64>) -> String {
    match cdf {
        Some(c) if (0.0..=1.0).contains(&c) => format!("{:.1}%", c * 100.0),
        Some(c) if c > 1.0 => format!("{:.1}%", c), // Already a percentage
        _ => "—".to_string(),
    }
}

/// Format peer distribution as a compact visual bar
/// Shows hot/warm/cold distribution: [████▒▒░░░░] H:5 W:3 C:10
fn format_peer_distribution(hot: Option<u64>, warm: Option<u64>, cold: Option<u64>) -> String {
    let h = hot.unwrap_or(0);
    let w = warm.unwrap_or(0);
    let c = cold.unwrap_or(0);
    let total = h + w + c;

    if total == 0 {
        return "—".to_string();
    }

    // Scale to 10 characters max
    let bar_width: usize = 10;
    let hot_chars = ((h as f64 / total as f64) * bar_width as f64).round() as usize;
    let warm_chars = ((w as f64 / total as f64) * bar_width as f64).round() as usize;
    let cold_chars = bar_width.saturating_sub(hot_chars + warm_chars);

    // Use block characters: █ (hot), ▒ (warm), ░ (cold)
    let bar: String = "█".repeat(hot_chars) + &"▒".repeat(warm_chars) + &"░".repeat(cold_chars);

    format!("[{}] H:{} W:{} C:{}", bar, h, w, c)
}
