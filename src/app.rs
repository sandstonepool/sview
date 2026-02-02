//! Application state management
//!
//! This module contains the core application state and logic.

use crate::config::Config;
use crate::metrics::{MetricsClient, NodeMetrics};
use std::time::Instant;

/// Main application state
pub struct App {
    /// Application configuration
    pub config: Config,
    /// Metrics client for fetching data
    metrics_client: MetricsClient,
    /// Current node metrics
    pub metrics: NodeMetrics,
    /// Last successful fetch time
    pub last_fetch: Option<Instant>,
    /// Last fetch error (if any)
    pub last_error: Option<String>,
    /// Time since last refresh
    last_refresh: Instant,
}

impl App {
    /// Create a new application instance
    pub fn new(config: Config) -> Self {
        let metrics_client = MetricsClient::new(config.metrics_url(), config.prom_timeout);

        Self {
            config,
            metrics_client,
            metrics: NodeMetrics::default(),
            last_fetch: None,
            last_error: None,
            last_refresh: Instant::now(),
        }
    }

    /// Fetch metrics from the node
    pub async fn fetch_metrics(&mut self) {
        match self.metrics_client.fetch().await {
            Ok(metrics) => {
                self.metrics = metrics;
                self.last_fetch = Some(Instant::now());
                self.last_error = None;
            }
            Err(e) => {
                self.metrics.connected = false;
                self.last_error = Some(e.to_string());
            }
        }
    }

    /// Called on each tick to handle periodic updates
    pub async fn tick(&mut self) {
        if self.last_refresh.elapsed() >= self.config.refresh_interval {
            self.fetch_metrics().await;
            self.last_refresh = Instant::now();
        }
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
