//! User interface rendering
//!
//! This module handles all TUI rendering using ratatui.

use crate::app::{App, AppMode, HealthStatus};
use crate::themes::Palette;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, Tabs, Wrap},
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

    // Draw main content area
    draw_main_content(frame, main_area, app, &palette);

    // Draw footer
    draw_footer(frame, footer_area, app, &palette);

    // Draw help overlay if in help mode
    if app.mode == AppMode::Help {
        draw_help_popup(frame, area, app.is_multi_node(), &palette);
    }

    // Draw peers overlay if in peers mode
    if app.mode == AppMode::Peers {
        draw_peers_view(frame, area, app, &palette);
    }

    // Draw peer detail overlay if in peer detail mode
    if app.mode == AppMode::PeerDetail {
        draw_peer_detail_view(frame, area, app, &palette);
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

    // Check for critical alerts
    let alert_span = if let Some(alert) = node.alert_manager.latest_critical() {
        vec![
            Span::raw("  │  "),
            Span::styled(
                format!("⚠ {} ", alert.title),
                Style::default().fg(palette.critical).bold(),
            ),
        ]
    } else {
        vec![]
    };

    let mut header_spans = vec![
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
        Span::styled(" Sync  ", Style::default().fg(palette.text_muted)),
        peer_dot,
        Span::styled(" Peers  ", Style::default().fg(palette.text_muted)),
        tip_dot,
        Span::styled(" Tip  ", Style::default().fg(palette.text_muted)),
        mem_dot,
        Span::styled(" Mem", Style::default().fg(palette.text_muted)),
    ];
    header_spans.extend(alert_span);

    let header_text = Line::from(header_spans);

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
    // 3 equal columns, each with gauge + metrics
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3), // Chain column
            Constraint::Ratio(1, 3), // Network column
            Constraint::Ratio(1, 3), // Resources column
        ])
        .split(area);

    draw_chain_column(frame, columns[0], app, palette);
    draw_network_column(frame, columns[1], app, palette);
    draw_resources_column(frame, columns[2], app, palette);
}

/// Draw chain column (epoch gauge + chain metrics)
fn draw_chain_column(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Epoch gauge
            Constraint::Min(5),    // Chain metrics
        ])
        .split(area);

    // Epoch progress gauge
    let node = app.current_node();
    let progress = node.epoch_progress().unwrap_or(0.0);
    let time_remaining = node.epoch_time_remaining();

    let label = match (node.metrics.epoch, time_remaining) {
        (Some(epoch), Some(secs)) => format!(
            "E{} {:.1}% {}",
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
                .title(" Epoch ")
                .border_style(Style::default().fg(palette.border)),
        )
        .gauge_style(Style::default().fg(gauge_color).bg(palette.gauge_bg))
        .ratio(progress / 100.0)
        .label(Span::styled(
            label,
            Style::default().fg(palette.gauge_label).bold(),
        ));

    frame.render_widget(gauge, chunks[0]);

    // Chain metrics
    draw_chain_metrics(frame, chunks[1], app, palette);
}

/// Draw network column (sync gauge + network metrics)
fn draw_network_column(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Sync gauge
            Constraint::Min(5),    // Network metrics
        ])
        .split(area);

    // Sync progress gauge
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
                .bg(palette.gauge_bg),
        )
        .ratio((progress / 100.0).min(1.0))
        .label(Span::styled(
            label,
            Style::default().fg(palette.gauge_label).bold(),
        ));

    frame.render_widget(gauge, chunks[0]);

    // Network metrics
    draw_network_metrics(frame, chunks[1], app, palette);
}

/// Draw resources column (memory gauge + resource metrics)
fn draw_resources_column(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Memory gauge
            Constraint::Min(5),    // Resource metrics
        ])
        .split(area);

    // Memory usage gauge
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
                .bg(palette.gauge_bg),
        )
        .ratio(ratio)
        .label(Span::styled(
            label,
            Style::default().fg(palette.gauge_label).bold(),
        ));

    frame.render_widget(gauge, chunks[0]);

    // Resource metrics
    draw_resource_metrics(frame, chunks[1], app, palette);
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
            "Peer Dist",
            format_peer_distribution(
                metrics.p2p.hot_peers,
                metrics.p2p.warm_peers,
                metrics.p2p.cold_peers,
            ),
            palette,
        ),
        create_metric_row(
            "Block Delay",
            format_block_delay(metrics.block_delay_s),
            palette,
        ),
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
        Span::styled("p", Style::default().fg(palette.tertiary)),
        Span::raw(" peers "),
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
            Span::styled("  p         ", Style::default().fg(palette.tertiary)),
            Span::raw("Show peer connections"),
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
// Peers view
// ============================================================================

/// Draw the detailed peer list view
fn draw_peers_view(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    // Use 90% of screen for peer list
    let popup_area = centered_rect(90, 85, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let node = app.current_node();
    let peers = &node.peer_connections;

    // Build table rows
    let mut rows: Vec<Row> = Vec::new();

    // Sort peers: incoming first, then by RTT
    let mut sorted_peers = peers.clone();
    sorted_peers.sort_by(|a, b| {
        // Sort by direction first (incoming first)
        match (a.incoming, b.incoming) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                // Then by RTT (lower is better)
                let a_rtt = a.rtt_ms.unwrap_or(f64::MAX);
                let b_rtt = b.rtt_ms.unwrap_or(f64::MAX);
                a_rtt
                    .partial_cmp(&b_rtt)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        }
    });

    if sorted_peers.is_empty() {
        rows.push(Row::new(vec![
            Cell::from(""),
            Cell::from(Span::styled(
                "No peer connections found. Press 'r' to refresh.",
                Style::default().fg(palette.text_muted).italic(),
            )),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ]));
    } else {
        for (idx, peer) in sorted_peers.iter().enumerate() {
            let is_selected = idx == app.peer_list_selected;

            let dir_style = if peer.incoming {
                Style::default().fg(palette.primary)
            } else {
                Style::default().fg(palette.secondary)
            };

            let rtt_str = match peer.rtt_ms {
                Some(rtt) if rtt < 50.0 => format!("{:.1}ms", rtt),
                Some(rtt) if rtt < 100.0 => format!("{:.1}ms", rtt),
                Some(rtt) => format!("{:.0}ms", rtt),
                None => "—".to_string(),
            };

            let rtt_style = match peer.rtt_ms {
                Some(rtt) if rtt < 50.0 => Style::default().fg(palette.healthy),
                Some(rtt) if rtt < 100.0 => Style::default().fg(palette.warning),
                Some(_) => Style::default().fg(palette.critical),
                None => Style::default().fg(palette.text_muted),
            };

            let queue_str = if peer.recv_q > 0 || peer.send_q > 0 {
                format!("R:{} S:{}", peer.recv_q, peer.send_q)
            } else {
                "0".to_string()
            };

            // Get location from cache
            let location = app
                .peer_locations
                .get(&peer.ip)
                .cloned()
                .unwrap_or_else(|| "—".to_string());

            // Selection indicator
            let selector = if is_selected { "▶" } else { " " };

            let mut row = Row::new(vec![
                Cell::from(Span::styled(selector, Style::default().fg(palette.primary))),
                Cell::from(Span::styled(peer.direction_str().to_string(), dir_style)),
                Cell::from(Span::styled(
                    peer.ip.clone(),
                    Style::default().fg(palette.text),
                )),
                Cell::from(Span::styled(
                    peer.port.to_string(),
                    Style::default().fg(palette.text_muted),
                )),
                Cell::from(Span::styled(
                    location,
                    Style::default().fg(palette.tertiary),
                )),
                Cell::from(Span::styled(rtt_str, rtt_style)),
                Cell::from(Span::styled(
                    queue_str,
                    Style::default().fg(palette.text_muted),
                )),
            ]);

            // Highlight selected row
            if is_selected {
                row = row.style(Style::default().bg(palette.gauge_bg));
            }

            rows.push(row);
        }
    }

    // Summary line
    let incoming_count = peers.iter().filter(|p| p.incoming).count();
    let outgoing_count = peers.iter().filter(|p| !p.incoming).count();
    let avg_rtt: f64 = {
        let rtts: Vec<f64> = peers.iter().filter_map(|p| p.rtt_ms).collect();
        if rtts.is_empty() {
            0.0
        } else {
            rtts.iter().sum::<f64>() / rtts.len() as f64
        }
    };

    let title = format!(
        " Peer Connections — {} total (IN: {} OUT: {}) — Avg RTT: {:.1}ms ",
        peers.len(),
        incoming_count,
        outgoing_count,
        avg_rtt
    );

    // Create header row
    let header = Row::new(vec![
        Cell::from(Span::styled(" ", Style::default())),
        Cell::from(Span::styled(
            "DIR",
            Style::default().fg(palette.primary).bold(),
        )),
        Cell::from(Span::styled(
            "IP ADDRESS",
            Style::default().fg(palette.primary).bold(),
        )),
        Cell::from(Span::styled(
            "PORT",
            Style::default().fg(palette.primary).bold(),
        )),
        Cell::from(Span::styled(
            "LOCATION",
            Style::default().fg(palette.primary).bold(),
        )),
        Cell::from(Span::styled(
            "RTT",
            Style::default().fg(palette.primary).bold(),
        )),
        Cell::from(Span::styled(
            "QUEUE",
            Style::default().fg(palette.primary).bold(),
        )),
    ])
    .style(Style::default())
    .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),  // Selection
            Constraint::Length(4),  // DIR
            Constraint::Min(15),    // IP
            Constraint::Length(6),  // PORT
            Constraint::Length(16), // LOCATION
            Constraint::Length(10), // RTT
            Constraint::Length(10), // QUEUE
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_bottom(
                Line::from(" [↑↓] select | [Enter] details | [p/Esc] close | [r] refresh ")
                    .centered(),
            )
            .border_style(Style::default().fg(palette.primary)),
    );

    frame.render_widget(table, popup_area);
}

/// Draw detailed view for a single selected peer
fn draw_peer_detail_view(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
    let popup_area = centered_rect(70, 60, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let peer = match app.selected_peer() {
        Some(p) => p,
        None => {
            let msg = Paragraph::new("No peer selected").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Peer Details ")
                    .border_style(Style::default().fg(palette.primary)),
            );
            frame.render_widget(msg, popup_area);
            return;
        }
    };

    // Get location
    let location = app
        .peer_locations
        .get(&peer.ip)
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    // Build detail lines
    let mut lines = vec![
        Line::from(Span::styled(
            "Connection Details",
            Style::default().bold().underlined().fg(palette.primary),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  IP Address:    ", Style::default().fg(palette.text_muted)),
            Span::styled(&peer.ip, Style::default().fg(palette.text).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Port:          ", Style::default().fg(palette.text_muted)),
            Span::styled(peer.port.to_string(), Style::default().fg(palette.text)),
        ]),
        Line::from(vec![
            Span::styled("  Location:      ", Style::default().fg(palette.text_muted)),
            Span::styled(location, Style::default().fg(palette.tertiary)),
        ]),
        Line::from(vec![
            Span::styled("  Direction:     ", Style::default().fg(palette.text_muted)),
            Span::styled(
                if peer.incoming {
                    "Incoming"
                } else {
                    "Outgoing"
                },
                Style::default().fg(if peer.incoming {
                    palette.primary
                } else {
                    palette.secondary
                }),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Performance",
            Style::default().bold().underlined().fg(palette.primary),
        )),
        Line::from(""),
    ];

    // RTT with color coding
    let (rtt_str, rtt_color) = match peer.rtt_ms {
        Some(rtt) if rtt < 50.0 => (format!("{:.2} ms (Excellent)", rtt), palette.healthy),
        Some(rtt) if rtt < 100.0 => (format!("{:.2} ms (Good)", rtt), palette.warning),
        Some(rtt) if rtt < 200.0 => (format!("{:.2} ms (Fair)", rtt), palette.warning),
        Some(rtt) => (format!("{:.2} ms (Poor)", rtt), palette.critical),
        None => ("Not available".to_string(), palette.text_muted),
    };

    lines.push(Line::from(vec![
        Span::styled("  RTT Latency:   ", Style::default().fg(palette.text_muted)),
        Span::styled(rtt_str, Style::default().fg(rtt_color)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  State:         ", Style::default().fg(palette.text_muted)),
        Span::styled(&peer.state, Style::default().fg(palette.healthy)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Buffers",
        Style::default().bold().underlined().fg(palette.primary),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  Receive Queue: ", Style::default().fg(palette.text_muted)),
        Span::styled(
            format!("{} bytes", peer.recv_q),
            Style::default().fg(if peer.recv_q > 0 {
                palette.warning
            } else {
                palette.text
            }),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Send Queue:    ", Style::default().fg(palette.text_muted)),
        Span::styled(
            format!("{} bytes", peer.send_q),
            Style::default().fg(if peer.send_q > 0 {
                palette.warning
            } else {
                palette.text
            }),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press [Backspace] or [←] to go back",
        Style::default().fg(palette.text_muted).italic(),
    )));

    let detail = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Peer: {} ", peer.ip))
                .border_style(Style::default().fg(palette.primary)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(detail, popup_area);
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
