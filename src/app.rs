//! Application state management
//!
//! This module contains the core application state and logic.

use crate::config::Config;
use crate::history::MetricsHistory;
use crate::metrics::{MetricsClient, NodeMetrics};
use std::time::Instant;

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

/// Main application state
pub struct App {
    /// Application configuration
    pub config: Config,
    /// Metrics client for fetching data
    metrics_client: MetricsClient,
    /// Current node metrics
    pub metrics: NodeMetrics,
    /// Historical metrics for sparklines
    pub history: MetricsHistory,
    /// Last successful fetch time
    pub last_fetch: Option<Instant>,
    /// Last fetch error (if any)
    pub last_error: Option<String>,
    /// Time since last refresh
    last_refresh: Instant,
    /// Current UI mode
    pub mode: AppMode,
    /// Fetch count (for tracking uptime)
    pub fetch_count: u64,
    /// Last observed block height (for tip age tracking)
    last_block_height: Option<u64>,
    /// Time when block height last changed
    last_block_time: Option<Instant>,
}

impl App {
    /// Create a new application instance
    pub fn new(config: Config) -> Self {
        let metrics_client = MetricsClient::new(config.metrics_url(), config.prom_timeout());
        let history = MetricsHistory::new(config.history_length);

        Self {
            config,
            metrics_client,
            metrics: NodeMetrics::default(),
            history,
            last_fetch: None,
            last_error: None,
            last_refresh: Instant::now(),
            mode: AppMode::Normal,
            fetch_count: 0,
            last_block_height: None,
            last_block_time: None,
        }
    }

    /// Fetch metrics from the node
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
                self.last_fetch = Some(Instant::now());
                self.last_error = None;
                self.fetch_count += 1;
            }
            Err(e) => {
                self.metrics.connected = false;
                self.last_error = Some(e.to_string());
            }
        }
    }

    /// Called on each tick to handle periodic updates
    pub async fn tick(&mut self) {
        if self.last_refresh.elapsed() >= self.config.refresh_interval() {
            self.fetch_metrics().await;
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
            // Warn at 12GB, critical at 14GB (typical cardano-node thresholds)
            Some(bytes) if bytes < 12_000_000_000 => HealthStatus::Good,
            Some(bytes) if bytes < 14_000_000_000 => HealthStatus::Warning,
            Some(_) => HealthStatus::Critical,
            None => HealthStatus::Good, // Unknown is fine
        }
    }

    /// Get the health status for KES key expiry
    pub fn kes_health(&self) -> HealthStatus {
        match self.metrics.kes_remaining {
            // Each KES period is ~1.5 days on mainnet (129600 slots / 86400 slots per day)
            // Warn at <20 periods (~30 days), critical at <5 periods (~7 days)
            Some(remaining) if remaining >= 20 => HealthStatus::Good,
            Some(remaining) if remaining >= 5 => HealthStatus::Warning,
            Some(_) => HealthStatus::Critical,
            None => HealthStatus::Good, // Not a block producer or unavailable
        }
    }

    /// Get seconds since last block was received
    pub fn tip_age_secs(&self) -> Option<u64> {
        self.last_block_time
            .map(|t| t.elapsed().as_secs())
    }

    /// Get the health status for tip age
    pub fn tip_health(&self) -> HealthStatus {
        match self.tip_age_secs() {
            // Mainnet produces blocks every ~20 seconds on average
            // Warn if no block for >60s, critical if >120s
            Some(age) if age < 60 => HealthStatus::Good,
            Some(age) if age < 120 => HealthStatus::Warning,
            Some(_) => HealthStatus::Critical,
            None => HealthStatus::Good, // No data yet
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

    /// Get blocks per minute from history (if enough data)
    pub fn blocks_per_minute(&self) -> Option<f64> {
        let trend = self.history.block_height.trend()?;
        let samples = self.history.block_height.len();
        if samples < 2 {
            return None;
        }
        // Calculate based on refresh interval and number of samples
        let seconds = samples as f64 * self.config.refresh_interval_secs as f64;
        Some(trend / seconds * 60.0)
    }
}
