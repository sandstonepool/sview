//! Historical metrics storage for sparkline visualization
//!
//! This module provides a ring buffer for storing historical metric values,
//! used to generate sparkline visualizations in the TUI.

use std::collections::VecDeque;

/// A ring buffer for storing historical metric values
#[derive(Debug, Clone)]
pub struct MetricHistory {
    /// Maximum number of data points to keep
    capacity: usize,
    /// Stored values
    values: VecDeque<f64>,
}

impl MetricHistory {
    /// Create a new metric history with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            values: VecDeque::with_capacity(capacity),
        }
    }

    /// Add a new value to the history
    pub fn push(&mut self, value: f64) {
        if self.values.len() >= self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    /// Get the values as a slice for sparkline rendering
    pub fn as_slice(&self) -> Vec<u64> {
        self.values.iter().map(|v| *v as u64).collect()
    }

    /// Get the current (most recent) value
    #[allow(dead_code)]
    pub fn current(&self) -> Option<f64> {
        self.values.back().copied()
    }

    /// Get the minimum value in the history
    pub fn min(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::min)
    }

    /// Get the maximum value in the history
    pub fn max(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::max)
    }

    /// Get the average value in the history
    #[allow(dead_code)]
    pub fn avg(&self) -> Option<f64> {
        if self.values.is_empty() {
            None
        } else {
            Some(self.values.iter().sum::<f64>() / self.values.len() as f64)
        }
    }

    /// Get the trend (difference between current and oldest)
    pub fn trend(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let oldest = self.values.front()?;
        let newest = self.values.back()?;
        Some(newest - oldest)
    }

    /// Get the number of stored values
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the history is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Clear all stored values
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.values.clear();
    }
}

/// Collection of metric histories for all tracked metrics
#[derive(Debug, Clone)]
pub struct MetricsHistory {
    pub block_height: MetricHistory,
    pub slot_num: MetricHistory,
    pub peers_connected: MetricHistory,
    pub memory_used: MetricHistory,
    pub mempool_txs: MetricHistory,
    pub sync_progress: MetricHistory,
    // P2P metrics
    pub p2p_hot_peers: MetricHistory,
    pub p2p_warm_peers: MetricHistory,
    pub p2p_cold_peers: MetricHistory,
}

impl MetricsHistory {
    /// Create a new metrics history collection with the given capacity per metric
    pub fn new(capacity: usize) -> Self {
        Self {
            block_height: MetricHistory::new(capacity),
            slot_num: MetricHistory::new(capacity),
            peers_connected: MetricHistory::new(capacity),
            memory_used: MetricHistory::new(capacity),
            mempool_txs: MetricHistory::new(capacity),
            sync_progress: MetricHistory::new(capacity),
            p2p_hot_peers: MetricHistory::new(capacity),
            p2p_warm_peers: MetricHistory::new(capacity),
            p2p_cold_peers: MetricHistory::new(capacity),
        }
    }

    /// Update all histories with new metric values
    pub fn update(&mut self, metrics: &crate::metrics::NodeMetrics) {
        if let Some(v) = metrics.block_height {
            self.block_height.push(v as f64);
        }
        if let Some(v) = metrics.slot_num {
            self.slot_num.push(v as f64);
        }
        if let Some(v) = metrics.peers_connected {
            self.peers_connected.push(v as f64);
        }
        if let Some(v) = metrics.memory_used {
            self.memory_used.push(v as f64);
        }
        if let Some(v) = metrics.mempool_txs {
            self.mempool_txs.push(v as f64);
        }
        if let Some(v) = metrics.sync_progress {
            self.sync_progress.push(v);
        }
        // P2P metrics
        if let Some(v) = metrics.p2p.hot_peers {
            self.p2p_hot_peers.push(v as f64);
        }
        if let Some(v) = metrics.p2p.warm_peers {
            self.p2p_warm_peers.push(v as f64);
        }
        if let Some(v) = metrics.p2p.cold_peers {
            self.p2p_cold_peers.push(v as f64);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_history_push() {
        let mut history = MetricHistory::new(5);
        for i in 1..=10 {
            history.push(i as f64);
        }
        assert_eq!(history.len(), 5);
        assert_eq!(history.current(), Some(10.0));
    }

    #[test]
    fn test_metric_history_trend() {
        let mut history = MetricHistory::new(5);
        history.push(10.0);
        history.push(15.0);
        history.push(20.0);
        assert_eq!(history.trend(), Some(10.0));
    }

    #[test]
    fn test_metric_history_stats() {
        let mut history = MetricHistory::new(5);
        history.push(10.0);
        history.push(20.0);
        history.push(30.0);
        assert_eq!(history.min(), Some(10.0));
        assert_eq!(history.max(), Some(30.0));
        assert_eq!(history.avg(), Some(20.0));
    }
}
