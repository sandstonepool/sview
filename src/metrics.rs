//! Prometheus metrics fetching and parsing
//!
//! This module handles connecting to a Cardano node's Prometheus endpoint
//! and parsing the metrics into structured data.

use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;

/// Detected node implementation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeType {
    #[default]
    CardanoNode,
    Dingo,
    Amaru,
    Unknown,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::CardanoNode => write!(f, "cardano-node"),
            NodeType::Dingo => write!(f, "Dingo"),
            NodeType::Amaru => write!(f, "Amaru"),
            NodeType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Parsed metrics from a Cardano node
#[derive(Debug, Clone, Default)]
pub struct NodeMetrics {
    /// Detected node type
    pub node_type: NodeType,
    /// Current block height
    pub block_height: Option<u64>,
    /// Current slot number
    pub slot_num: Option<u64>,
    /// Current epoch
    pub epoch: Option<u64>,
    /// Slot in epoch
    pub slot_in_epoch: Option<u64>,
    /// Number of connected peers
    pub peers_connected: Option<u64>,
    /// Memory usage in bytes
    pub memory_used: Option<u64>,
    /// CPU usage percentage (if available)
    pub cpu_seconds: Option<f64>,
    /// Node uptime in seconds
    #[allow(dead_code)]
    pub uptime_seconds: Option<f64>,
    /// Transactions in mempool
    pub mempool_txs: Option<u64>,
    /// Mempool bytes
    pub mempool_bytes: Option<u64>,
    /// Node version (if available)
    #[allow(dead_code)]
    pub version: Option<String>,
    /// Sync progress percentage (0-100)
    pub sync_progress: Option<f64>,
    /// Whether we successfully connected to the node
    pub connected: bool,
    /// Raw metrics for debugging/advanced display
    pub raw: HashMap<String, f64>,
}

/// Metrics client for fetching Prometheus data
pub struct MetricsClient {
    client: reqwest::Client,
    url: String,
}

impl MetricsClient {
    /// Create a new metrics client
    pub fn new(url: String, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, url }
    }

    /// Fetch and parse metrics from the node
    pub async fn fetch(&self) -> Result<NodeMetrics> {
        let response = self.client.get(&self.url).send().await?;
        let text = response.text().await?;
        Ok(parse_prometheus_metrics(&text))
    }
}

/// Parse Prometheus text format into NodeMetrics
fn parse_prometheus_metrics(text: &str) -> NodeMetrics {
    let mut metrics = NodeMetrics {
        connected: true,
        ..Default::default()
    };

    for line in text.lines() {
        // Skip comments and empty lines
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        // Parse metric line: metric_name{labels} value
        if let Some((name, value)) = parse_metric_line(line) {
            metrics.raw.insert(name.clone(), value);

            // Map known metrics to structured fields
            match name.as_str() {
                // Block/Chain metrics
                "cardano_node_metrics_blockNum_int" => {
                    metrics.block_height = Some(value as u64);
                }
                "cardano_node_metrics_slotNum_int" => {
                    metrics.slot_num = Some(value as u64);
                }
                "cardano_node_metrics_epoch_int" => {
                    metrics.epoch = Some(value as u64);
                }
                "cardano_node_metrics_slotInEpoch_int" => {
                    metrics.slot_in_epoch = Some(value as u64);
                }

                // Peer metrics
                "cardano_node_metrics_connectedPeers_int" => {
                    metrics.peers_connected = Some(value as u64);
                }

                // Resource metrics
                "cardano_node_metrics_RTS_gcLiveBytes_int" => {
                    metrics.memory_used = Some(value as u64);
                }
                "cardano_node_metrics_RTS_cpuNs_int" => {
                    // Convert nanoseconds to seconds
                    metrics.cpu_seconds = Some(value / 1_000_000_000.0);
                }

                // Mempool metrics
                "cardano_node_metrics_txsInMempool_int" => {
                    metrics.mempool_txs = Some(value as u64);
                }
                "cardano_node_metrics_mempoolBytes_int" => {
                    metrics.mempool_bytes = Some(value as u64);
                }

                // Sync progress (if available)
                "cardano_node_metrics_ChainSync_progress" => {
                    metrics.sync_progress = Some(value * 100.0);
                }

                _ => {}
            }
        }
    }

    // Detect node type based on available metrics
    metrics.node_type = detect_node_type(&metrics.raw);

    metrics
}

/// Parse a single Prometheus metric line
fn parse_metric_line(line: &str) -> Option<(String, f64)> {
    // Handle lines with labels: metric_name{label="value"} 123.45
    // And simple lines: metric_name 123.45

    let line = line.trim();

    // Find the metric name (everything before '{' or ' ')
    let name_end = line.find('{').or_else(|| line.find(' '))?;
    let name = line[..name_end].to_string();

    // Find the value (last space-separated element)
    let value_str = line.rsplit_once(' ')?.1;
    let value: f64 = value_str.parse().ok()?;

    Some((name, value))
}

/// Detect the node implementation type based on available metrics
fn detect_node_type(metrics: &HashMap<String, f64>) -> NodeType {
    // Check for Dingo-specific metrics
    if metrics.keys().any(|k| k.starts_with("dingo_")) {
        return NodeType::Dingo;
    }

    // Check for Amaru-specific metrics
    if metrics.keys().any(|k| k.starts_with("amaru_")) {
        return NodeType::Amaru;
    }

    // Check for standard cardano-node metrics
    if metrics.keys().any(|k| k.starts_with("cardano_node_")) {
        return NodeType::CardanoNode;
    }

    NodeType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metric_line_simple() {
        let (name, value) = parse_metric_line("cardano_node_metrics_blockNum_int 12345").unwrap();
        assert_eq!(name, "cardano_node_metrics_blockNum_int");
        assert_eq!(value, 12345.0);
    }

    #[test]
    fn test_parse_metric_line_with_labels() {
        let (name, value) = parse_metric_line("http_requests_total{method=\"GET\"} 1234").unwrap();
        assert_eq!(name, "http_requests_total");
        assert_eq!(value, 1234.0);
    }

    #[test]
    fn test_parse_prometheus_metrics() {
        let text = r#"
# HELP cardano_node_metrics_blockNum_int Block number
# TYPE cardano_node_metrics_blockNum_int gauge
cardano_node_metrics_blockNum_int 10500000
cardano_node_metrics_slotNum_int 125000000
cardano_node_metrics_epoch_int 450
cardano_node_metrics_connectedPeers_int 5
"#;
        let metrics = parse_prometheus_metrics(text);
        assert_eq!(metrics.block_height, Some(10500000));
        assert_eq!(metrics.slot_num, Some(125000000));
        assert_eq!(metrics.epoch, Some(450));
        assert_eq!(metrics.peers_connected, Some(5));
        assert_eq!(metrics.node_type, NodeType::CardanoNode);
    }
}
