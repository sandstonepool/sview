//! Application state management
//!
//! This module contains the core application state and logic.
//! Supports both single-node and multi-node monitoring modes.

use crate::config::{AppConfig, Config, NodeRole, NodeRuntimeConfig};
use crate::history::MetricsHistory;
use crate::metrics::{MetricsClient, NodeMetrics};
use crate::storage::StorageManager;
use std::time::Instant;
use tracing::{debug, warn};

/// UI mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    Normal,
    Help,
}

/// Health status indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Good,
    Warning,
    Critical,
}

/// State for a single monitored node
pub struct NodeState {
    /// Node configuration
    pub config: Config,
    /// Node role (for display hints)
    pub role: NodeRole,
    /// Metrics client for fetching data
    metrics_client: MetricsClient,
    /// Current node metrics
    pub metrics: NodeMetrics,
    /// Historical metrics for sparklines
    pub history: MetricsHistory,
    /// Persistent storage manager
    storage: StorageManager,
    /// Last fetch error (if any)
    pub last_error: Option<String>,
    /// Fetch count
    pub fetch_count: u64,
    /// Last observed block height (for tip age tracking)
    last_block_height: Option<u64>,
    /// Time when block height last changed
    last_block_time: Option<Instant>,
}

impl NodeState {
    /// Create a new node state
    pub fn new(node_config: &NodeRuntimeConfig, app_config: &AppConfig) -> Self {
        let config = Config::from_node(node_config, app_config);
        let metrics_client = MetricsClient::new(config.metrics_url(), config.prom_timeout());
        let mut history = MetricsHistory::new(config.history_length);

        // Initialize storage and load historical data
        let storage = StorageManager::new(&config.node_name);

        // Try to load historical data to backfill sparklines
        match storage.populate_history(&mut history, config.history_length) {
            Ok(()) => {
                debug!(
                    "Loaded historical data for '{}' ({} samples)",
                    config.node_name,
                    history.block_height.len()
                );
            }
            Err(e) => {
                debug!("No historical data loaded for '{}': {}", config.node_name, e);
            }
        }

        // Run periodic cleanup of old data
        if let Err(e) = storage.cleanup_old_data() {
            warn!("Failed to cleanup old data for '{}': {}", config.node_name, e);
        }

        Self {
            config,
            role: node_config.role,
            metrics_client,
            metrics: NodeMetrics::default(),
            history,
            storage,
            last_error: None,
            fetch_count: 0,
            last_block_height: None,
            last_block_time: None,
        }
    }

    /// Fetch metrics from this node
    pub async fn fetch_metrics(&mut self) {
        match self.metrics_client.fetch().await {
            Ok(metrics) => {
                // Track tip age: detect when block height changes
                if let Some(new_height) = metrics.block_height {
                    let height_changed = self
                        .last_block_height
                        .map(|old| old != new_height)
                        .unwrap_or(true);

                    if height_changed {
                        self.last_block_height = Some(new_height);
                        self.last_block_time = Some(Instant::now());
                    }
                }

                self.metrics = metrics;
                self.history.update(&self.metrics);
                self.last_error = None;
                self.fetch_count += 1;

                // Save snapshot to persistent storage (hourly sampling)
                if let Err(e) = self.storage.save_snapshot(&self.metrics) {
                    debug!("Failed to save metric snapshot: {}", e);
                }
            }
            Err(e) => {
                self.metrics.connected = false;
                self.last_error = Some(e.to_string());
            }
        }
    }

    /// Get the storage manager for this node
    pub fn storage(&self) -> &StorageManager {
        &self.storage
    }

    /// Get seconds since last block was received
    pub fn tip_age_secs(&self) -> Option<u64> {
        self.last_block_time.map(|t| t.elapsed().as_secs())
    }

    /// Get the health status for peer count
    pub fn peer_health(&self) -> HealthStatus {
        match self.metrics.peers_connected {
            Some(peers) if peers >= 5 => HealthStatus::Good,
            Some(peers) if peers >= 2 => HealthStatus::Warning,
            Some(_) => HealthStatus::Critical,
            None => HealthStatus::Warning,
        }
    }

    /// Get the health status for sync progress
    pub fn sync_health(&self) -> HealthStatus {
        match self.metrics.sync_progress {
            Some(progress) if progress >= 99.9 => HealthStatus::Good,
            Some(progress) if progress >= 95.0 => HealthStatus::Warning,
            Some(_) => HealthStatus::Critical,
            None => HealthStatus::Warning,
        }
    }

    /// Get the health status for memory usage
    pub fn memory_health(&self) -> HealthStatus {
        match self.metrics.memory_used {
            Some(bytes) if bytes < 12_000_000_000 => HealthStatus::Good,
            Some(bytes) if bytes < 14_000_000_000 => HealthStatus::Warning,
            Some(_) => HealthStatus::Critical,
            None => HealthStatus::Good,
        }
    }

    /// Get the health status for KES key expiry
    pub fn kes_health(&self) -> HealthStatus {
        match self.metrics.kes_remaining {
            Some(remaining) if remaining >= 20 => HealthStatus::Good,
            Some(remaining) if remaining >= 5 => HealthStatus::Warning,
            Some(_) => HealthStatus::Critical,
            None => HealthStatus::Good,
        }
    }

    /// Get the health status for tip age
    pub fn tip_health(&self) -> HealthStatus {
        match self.tip_age_secs() {
            Some(age) if age < 60 => HealthStatus::Good,
            Some(age) if age < 120 => HealthStatus::Warning,
            Some(_) => HealthStatus::Critical,
            None => HealthStatus::Good,
        }
    }

    /// Get the overall node health
    pub fn overall_health(&self) -> HealthStatus {
        if !self.metrics.connected {
            return HealthStatus::Critical;
        }

        let statuses = [
            self.peer_health(),
            self.sync_health(),
            self.memory_health(),
            self.kes_health(),
            self.tip_health(),
        ];

        if statuses.contains(&HealthStatus::Critical) {
            HealthStatus::Critical
        } else if statuses.contains(&HealthStatus::Warning) {
            HealthStatus::Warning
        } else {
            HealthStatus::Good
        }
    }

    /// Get blocks per minute from history
    pub fn blocks_per_minute(&self) -> Option<f64> {
        let trend = self.history.block_height.trend()?;
        let samples = self.history.block_height.len();
        if samples < 2 {
            return None;
        }
        let seconds = samples as f64 * self.config.refresh_interval_secs as f64;
        Some(trend / seconds * 60.0)
    }

    /// Get epoch progress as a percentage
    pub fn epoch_progress(&self) -> Option<f64> {
        let slot_in_epoch = self.metrics.slot_in_epoch? as f64;
        let epoch_length = self.config.epoch_length as f64;
        Some((slot_in_epoch / epoch_length) * 100.0)
    }

    /// Get estimated time remaining in the current epoch
    pub fn epoch_time_remaining(&self) -> Option<u64> {
        let slot_in_epoch = self.metrics.slot_in_epoch?;
        let remaining_slots = self.config.epoch_length.saturating_sub(slot_in_epoch);
        Some(remaining_slots)
    }

    /// Get the status text for display
    pub fn status_text(&self) -> &str {
        if self.metrics.connected {
            "Connected"
        } else if self.last_error.is_some() {
            "Connection Error"
        } else {
            "Connecting..."
        }
    }
}

/// Main application state supporting multiple nodes
pub struct App {
    /// Application configuration
    pub app_config: AppConfig,
    /// All monitored nodes
    pub nodes: Vec<NodeState>,
    /// Currently selected node index
    pub selected_node: usize,
    /// Time since last refresh
    last_refresh: Instant,
    /// Current UI mode
    pub mode: AppMode,
}

impl App {
    /// Create a new application instance
    pub fn new(app_config: AppConfig) -> Self {
        let nodes: Vec<NodeState> = app_config
            .nodes
            .iter()
            .map(|n| NodeState::new(n, &app_config))
            .collect();

        Self {
            app_config,
            nodes,
            selected_node: 0,
            last_refresh: Instant::now(),
            mode: AppMode::Normal,
        }
    }

    /// Get the currently selected node
    pub fn current_node(&self) -> &NodeState {
        &self.nodes[self.selected_node]
    }

    /// Get the currently selected node mutably
    #[allow(dead_code)]
    pub fn current_node_mut(&mut self) -> &mut NodeState {
        &mut self.nodes[self.selected_node]
    }

    /// Select the next node
    pub fn next_node(&mut self) {
        if self.nodes.len() > 1 {
            self.selected_node = (self.selected_node + 1) % self.nodes.len();
        }
    }

    /// Select the previous node
    pub fn prev_node(&mut self) {
        if self.nodes.len() > 1 {
            self.selected_node = if self.selected_node == 0 {
                self.nodes.len() - 1
            } else {
                self.selected_node - 1
            };
        }
    }

    /// Select a node by index (0-based, for keyboard shortcuts 1-9)
    pub fn select_node(&mut self, index: usize) {
        if index < self.nodes.len() {
            self.selected_node = index;
        }
    }

    /// Check if in multi-node mode
    pub fn is_multi_node(&self) -> bool {
        self.nodes.len() > 1
    }

    /// Fetch metrics from all nodes
    pub async fn fetch_all_metrics(&mut self) {
        for node in &mut self.nodes {
            node.fetch_metrics().await;
        }
    }

    /// Fetch metrics from the current node only
    #[allow(dead_code)]
    pub async fn fetch_current_metrics(&mut self) {
        self.nodes[self.selected_node].fetch_metrics().await;
    }

    /// Called on each tick to handle periodic updates
    pub async fn tick(&mut self) {
        if self.last_refresh.elapsed() >= self.app_config.refresh_interval {
            self.fetch_all_metrics().await;
            self.last_refresh = Instant::now();
        }
    }

    /// Toggle help mode
    pub fn toggle_help(&mut self) {
        self.mode = match self.mode {
            AppMode::Normal => AppMode::Help,
            AppMode::Help => AppMode::Normal,
        };
    }

}
