//! Prometheus metrics fetching and parsing
//!
//! This module handles connecting to a Cardano node's Prometheus endpoint
//! and parsing the metrics into structured data.

use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;
use tracing::debug;

/// P2P (peer-to-peer) network statistics
#[derive(Debug, Clone, Default)]
pub struct P2PStats {
    /// Whether P2P is enabled on this node
    pub enabled: Option<bool>,
    /// Number of cold peers (not yet known)
    pub cold_peers: Option<u64>,
    /// Number of warm peers (known but not actively used)
    pub warm_peers: Option<u64>,
    /// Number of hot peers (actively used)
    pub hot_peers: Option<u64>,
}

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

/// Parsed metrics from a Cardano node (matches nview PromMetrics)
#[derive(Debug, Clone, Default)]
pub struct NodeMetrics {
    /// Detected node type
    pub node_type: NodeType,
    /// Node software version (from build_info metric or config)
    /// Reserved for future Prometheus build_info parsing
    #[allow(dead_code)]
    pub node_version: Option<String>,
    /// Current block height
    pub block_height: Option<u64>,
    /// Current slot number
    pub slot_num: Option<u64>,
    /// Current epoch
    pub epoch: Option<u64>,
    /// Slot in epoch
    pub slot_in_epoch: Option<u64>,
    /// Chain density (real value, not percentage)
    pub density: Option<f64>,
    /// Transactions processed
    pub tx_processed: Option<u64>,
    /// Transactions in mempool
    pub mempool_txs: Option<u64>,
    /// Mempool bytes
    pub mempool_bytes: Option<u64>,
    /// Number of connected peers
    pub peers_connected: Option<u64>,
    /// Memory live (GC live bytes)
    pub memory_used: Option<u64>,
    /// Memory heap bytes
    pub memory_heap: Option<u64>,
    /// GC minor collections
    pub gc_minor: Option<u64>,
    /// GC major collections
    pub gc_major: Option<u64>,
    /// Number of forks
    pub forks: Option<u64>,
    /// Block fetch delay in seconds
    pub block_delay_s: Option<f64>,
    /// Blocks served
    pub blocks_served: Option<u64>,
    /// Blocks received late
    pub blocks_late: Option<u64>,
    /// Block delay CDF at 1s
    pub block_delay_cdf_1s: Option<f64>,
    /// Block delay CDF at 3s
    pub block_delay_cdf_3s: Option<f64>,
    /// Block delay CDF at 5s
    pub block_delay_cdf_5s: Option<f64>,
    /// CPU usage in milliseconds (from GC)
    pub cpu_ms: Option<u64>,
    /// Node uptime in seconds (calculated from nodeStartTime)
    pub uptime_seconds: Option<f64>,
    /// Sync progress percentage (0-100)
    pub sync_progress: Option<f64>,
    /// Whether we successfully connected to the node
    pub connected: bool,
    /// Raw metrics for debugging/advanced display
    pub raw: HashMap<String, f64>,
    // KES (Key Evolving Signature) metrics - critical for block producers
    /// Current KES period
    pub kes_period: Option<u64>,
    /// Remaining KES periods until expiry
    pub kes_remaining: Option<u64>,
    /// KES periods per operational certificate
    pub kes_periods_per_cert: Option<u64>,
    // Forging metrics (block producers)
    /// Whether forging/block production is enabled (0=relay, 1=BP)
    pub forging_enabled: Option<bool>,
    /// Is node a leader
    pub is_leader: Option<bool>,
    /// Blocks adopted by the node
    pub blocks_adopted: Option<u64>,
    /// Blocks not adopted
    pub blocks_didnt_adopt: Option<u64>,
    /// About to lead slots
    pub about_to_lead: Option<u64>,
    /// Missed slots
    pub missed_slots: Option<u64>,
    /// Operational certificate counter (on disk)
    pub op_cert_counter_disk: Option<u64>,
    /// Operational certificate counter (on chain)
    pub op_cert_counter_chain: Option<u64>,
    /// Operational certificate start KES period
    pub op_cert_start_kes_period: Option<u64>,
    /// P2P (peer-to-peer) network statistics
    pub p2p: P2PStats,
    /// Node start time (unix timestamp)
    pub node_start_time: Option<u64>,
    /// Incoming connections
    pub incoming_connections: Option<u64>,
    /// Outgoing connections
    pub outgoing_connections: Option<u64>,
    /// Full duplex connections (duplex conns from connectionManager)
    pub full_duplex_connections: Option<u64>,
    /// Unidirectional connections
    pub unidirectional_connections: Option<u64>,
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

            // Log interesting metrics for debugging
            if name.contains("Uptime")
                || name.contains("upTime")
                || name.contains("cpu")
                || name.contains("Mempool")
                || name.contains("Txs")
                || name.contains("blockdelay")
                || name.contains("cdf")
            {
                debug!("Found metric: {} = {}", name, value);
            }

            // Map known metrics to structured fields (matches nview PromMetrics names)
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
                "cardano_node_metrics_density_real" => {
                    metrics.density = Some(value);
                }
                // txsProcessedNum - various cardano-node versions use different suffixes
                "cardano_node_metrics_txsProcessedNum_int"
                | "cardano_node_metrics_txsProcessedNum_counter"
                | "cardano_node_metrics_txsProcessedNum" => {
                    metrics.tx_processed = Some(value as u64);
                }
                // forks - various cardano-node versions use different suffixes
                "cardano_node_metrics_forks_int"
                | "cardano_node_metrics_forks_counter"
                | "cardano_node_metrics_forks" => {
                    metrics.forks = Some(value as u64);
                }
                // slotsMissed naming varies between versions
                "cardano_node_metrics_slotsMissedNum_int"
                | "cardano_node_metrics_slotsMissed_int" => {
                    metrics.missed_slots = Some(value as u64);
                }

                // Peer metrics
                "cardano_node_metrics_connectedPeers_int" => {
                    metrics.peers_connected = Some(value as u64);
                }

                // Resource metrics (GC and memory)
                "cardano_node_metrics_RTS_gcLiveBytes_int" => {
                    metrics.memory_used = Some(value as u64);
                }
                "cardano_node_metrics_RTS_gcHeapBytes_int" => {
                    metrics.memory_heap = Some(value as u64);
                }
                "cardano_node_metrics_Mem_resident_int" => {
                    // Resident memory (fallback if GC metrics unavailable)
                    if metrics.memory_used.is_none() {
                        metrics.memory_used = Some(value as u64);
                    }
                }
                "cardano_node_metrics_RTS_gcMinorNum_int" => {
                    metrics.gc_minor = Some(value as u64);
                }
                "cardano_node_metrics_RTS_gcMajorNum_int" => {
                    metrics.gc_major = Some(value as u64);
                }
                // CPU metrics from GC
                "rts_gc_cpu_ms" => {
                    metrics.cpu_ms = Some(value as u64);
                }
                "cardano_node_metrics_RTS_cpuNs_int"
                | "cardano_node_metrics_RTS_cpu_ns"
                | "cardano_node_metrics_RTS_cpuNs" => {
                    // Convert nanoseconds to milliseconds
                    metrics.cpu_ms = Some((value / 1_000_000.0) as u64);
                }

                // Mempool metrics
                "cardano_node_metrics_txsInMempool_int" => {
                    metrics.mempool_txs = Some(value as u64);
                }
                "cardano_node_metrics_mempoolBytes_int" => {
                    metrics.mempool_bytes = Some(value as u64);
                }

                // Block fetch client metrics
                // blockdelay - from cardano-node BlockFetchClient metrics
                // Note: source emits as "blockfetchclient.blockdelay" which becomes
                // "cardano_node_metrics_blockfetchclient_blockdelay" (dots to underscores)
                // The _s suffix may be added by some exporters
                "cardano_node_metrics_blockfetchclient_blockdelay"
                | "cardano_node_metrics_blockfetchclient_blockdelay_s"
                | "cardano_node_metrics_blockfetchclient_blockdelay_real" => {
                    metrics.block_delay_s = Some(value);
                }
                // served.block can be _int (legacy) or _counter (current)
                "cardano_node_metrics_served_block_count_int"
                | "cardano_node_metrics_served_block_count_counter"
                | "cardano_node_metrics_served_block_counter"
                | "cardano_node_metrics_served_block_count" => {
                    metrics.blocks_served = Some(value as u64);
                }
                // lateblocks is a counter - emitted when delay > 5s
                "cardano_node_metrics_blockfetchclient_lateblocks"
                | "cardano_node_metrics_blockfetchclient_lateblocks_int"
                | "cardano_node_metrics_blockfetchclient_lateblocks_counter" => {
                    metrics.blocks_late = Some(value as u64);
                }
                // CDF metrics - calculated by cardano-node over sliding window
                // Only emitted after node receives 45+ blocks
                // Source: "blockfetchclient.blockdelay.cdfOne/Three/Five"
                // Values are fractions 0.0-1.0 (probability)
                "cardano_node_metrics_blockfetchclient_blockdelay_cdfOne"
                | "cardano_node_metrics_blockfetchclient_blockdelay_cdfOne_real" => {
                    metrics.block_delay_cdf_1s = Some(value);
                }
                "cardano_node_metrics_blockfetchclient_blockdelay_cdfThree"
                | "cardano_node_metrics_blockfetchclient_blockdelay_cdfThree_real" => {
                    metrics.block_delay_cdf_3s = Some(value);
                }
                "cardano_node_metrics_blockfetchclient_blockdelay_cdfFive"
                | "cardano_node_metrics_blockfetchclient_blockdelay_cdfFive_real" => {
                    metrics.block_delay_cdf_5s = Some(value);
                }

                // Uptime metrics
                // nodeStartTime vs node.start.time naming varies by cardano-node version
                "cardano_node_metrics_nodeStartTime_int"
                | "cardano_node_metrics_node_start_time_int" => {
                    metrics.node_start_time = Some(value as u64);
                }
                "cardano_node_metrics_upTime_ns" | "cardano_node_metrics_Stat_startTime" => {
                    // Convert nanoseconds to seconds
                    metrics.uptime_seconds = Some(value / 1_000_000_000.0);
                }

                // Connection manager metrics (official names from nview)
                "cardano_node_metrics_connectionManager_incomingConns" => {
                    metrics.incoming_connections = Some(value as u64);
                }
                "cardano_node_metrics_connectionManager_outgoingConns" => {
                    metrics.outgoing_connections = Some(value as u64);
                }
                "cardano_node_metrics_connectionManager_duplexConns" => {
                    metrics.full_duplex_connections = Some(value as u64);
                }
                "cardano_node_metrics_connectionManager_unidirectionalConns" => {
                    metrics.unidirectional_connections = Some(value as u64);
                }
                // Legacy fullDuplexConns name for compatibility
                "cardano_node_metrics_connectionManager_fullDuplexConns" => {
                    if metrics.full_duplex_connections.is_none() {
                        metrics.full_duplex_connections = Some(value as u64);
                    }
                }

                // P2P (peer-to-peer) network metrics
                "cardano_node_metrics_p2p_enabled_int" => {
                    metrics.p2p.enabled = Some(value > 0.0);
                }
                "cardano_node_metrics_p2p_coldPeersCount_int" => {
                    metrics.p2p.cold_peers = Some(value as u64);
                }
                "cardano_node_metrics_p2p_warmPeersCount_int" => {
                    metrics.p2p.warm_peers = Some(value as u64);
                }
                "cardano_node_metrics_p2p_hotPeersCount_int" => {
                    metrics.p2p.hot_peers = Some(value as u64);
                }

                // Peer selection metrics (CamelCase in current cardano-node)
                // Handle both lowercase (legacy) and CamelCase (current) variants
                "cardano_node_metrics_peerSelection_cold"
                | "cardano_node_metrics_peerSelection_Cold_int" => {
                    metrics.p2p.cold_peers = Some(value as u64);
                }
                "cardano_node_metrics_peerSelection_warm"
                | "cardano_node_metrics_peerSelection_Warm_int" => {
                    metrics.p2p.warm_peers = Some(value as u64);
                }
                "cardano_node_metrics_peerSelection_hot"
                | "cardano_node_metrics_peerSelection_Hot_int" => {
                    metrics.p2p.hot_peers = Some(value as u64);
                }

                // KES (Key Evolving Signature) metrics
                "cardano_node_metrics_currentKESPeriod_int" => {
                    if value >= 0.0 && value.is_finite() {
                        metrics.kes_period = Some(value as u64);
                    }
                }
                "cardano_node_metrics_remainingKESPeriods_int" => {
                    if value >= 0.0 && value.is_finite() {
                        metrics.kes_remaining = Some(value as u64);
                    } else if !value.is_finite() {
                        debug!("Invalid KES remaining value: {}", value);
                    }
                }
                "cardano_node_metrics_operationalCertificateExpiryKESPeriod_int" => {
                    if value >= 0.0 && value.is_finite() {
                        metrics.kes_periods_per_cert = Some(value as u64);
                    }
                }

                // Forging metrics (block producers)
                // forging_enabled: 0 = relay, 1 = block producer
                "cardano_node_metrics_forging_enabled_int" => {
                    metrics.forging_enabled = Some(value > 0.0);
                }
                "cardano_node_metrics_Forge_node_is_leader_int" => {
                    metrics.is_leader = Some(value > 0.0);
                }
                // blocksForged naming varies between ForgingStats and Forge tracers
                "cardano_node_metrics_Forge_adopted_int"
                | "cardano_node_metrics_blocksForged_int" => {
                    metrics.blocks_adopted = Some(value as u64);
                }
                "cardano_node_metrics_Forge_didnt_adopt_int" => {
                    metrics.blocks_didnt_adopt = Some(value as u64);
                }
                "cardano_node_metrics_Forge_forge_about_to_lead_int" => {
                    metrics.about_to_lead = Some(value as u64);
                }
                // nodeCannotForge and nodeIsLeader from ForgingStats
                "cardano_node_metrics_nodeIsLeader_int" => {
                    if metrics.is_leader.is_none() {
                        metrics.is_leader = Some(value > 0.0);
                    }
                }

                // Operational certificate metrics
                "cardano_node_metrics_operationalCertificateStartKESPeriod_int" => {
                    metrics.op_cert_start_kes_period = Some(value as u64);
                }
                // These may come from extended metrics or external tooling
                "cardano_node_metrics_opCertCounterOnDisk_int" => {
                    metrics.op_cert_counter_disk = Some(value as u64);
                }
                "cardano_node_metrics_opCertCounterOnChain_int" => {
                    metrics.op_cert_counter_chain = Some(value as u64);
                }

                // Log unrecognized cardano_node_metrics for debugging
                other if other.starts_with("cardano_node_metrics_") => {
                    debug!("Unrecognized metric: {} = {}", other, value);
                }

                _ => {}
            }
        }
    }

    // Detect node type based on available metrics
    metrics.node_type = detect_node_type(&metrics.raw);

    // Calculate uptime from nodeStartTime if available
    if let Some(start_time) = metrics.node_start_time {
        let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => dur.as_secs(),
            Err(_) => {
                debug!("System clock error during uptime calculation, skipping");
                0 // Skip uptime calculation on clock error
            }
        };
        if now >= start_time {
            metrics.uptime_seconds = Some((now - start_time) as f64);
        }
    }

    // Calculate peers_connected from peer states if direct metric is unavailable
    // Real Cardano nodes expose peerSelection_* metrics, not connectedPeers_int
    if metrics.peers_connected.is_none() {
        let cold = metrics.p2p.cold_peers.unwrap_or(0);
        let warm = metrics.p2p.warm_peers.unwrap_or(0);
        let hot = metrics.p2p.hot_peers.unwrap_or(0);

        if cold > 0 || warm > 0 || hot > 0 {
            metrics.peers_connected = Some(cold + warm + hot);
            debug!(
                "Calculated peers_connected from peer states: {} + {} + {} = {}",
                cold,
                warm,
                hot,
                metrics.peers_connected.unwrap_or(0)
            );
        }
    }

    // Calculate sync progress from slot number
    // Sync progress = (current_slot / expected_slot) * 100
    // Expected slot is calculated from time since network genesis
    if let Some(slot_num) = metrics.slot_num {
        let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => dur.as_secs(),
            Err(_) => 0,
        };

        if now > 0 {
            // Mainnet Byron genesis: 1506203091 (2017-09-23 21:44:51 UTC)
            // Slot length: 1 second (post-Shelley)
            // This is a simplified calculation - real sync depends on network params
            const MAINNET_GENESIS: u64 = 1506203091;
            const SHELLEY_TRANSITION_SLOT: u64 = 4492800; // Approximate slot at Shelley transition
            const SHELLEY_TRANSITION_TIME: u64 = 1596059091; // Byron slots were 20s, Shelley is 1s

            // Calculate expected slot
            let expected_slot = if now > SHELLEY_TRANSITION_TIME {
                // Post-Shelley: 1 slot per second
                let time_since_shelley = now - SHELLEY_TRANSITION_TIME;
                SHELLEY_TRANSITION_SLOT + time_since_shelley
            } else {
                // Byron era: 1 slot per 20 seconds
                (now - MAINNET_GENESIS) / 20
            };

            if expected_slot > 0 {
                let sync = (slot_num as f64 / expected_slot as f64) * 100.0;
                // Cap at 100% and ensure non-negative
                metrics.sync_progress = Some(sync.clamp(0.0, 100.0));
                debug!(
                    "Calculated sync_progress: slot {} / expected {} = {:.2}%",
                    slot_num,
                    expected_slot,
                    metrics.sync_progress.unwrap_or(0.0)
                );
            }
        }
    }

    // Log available metrics if in debug mode
    let available_metrics: Vec<&str> = metrics
        .raw
        .keys()
        .filter(|k| {
            k.contains("Uptime")
                || k.contains("upTime")
                || k.contains("cpu")
                || k.contains("Mempool")
                || k.contains("memory")
                || k.contains("Memory")
                || k.contains("connection")
                || k.contains("Connection")
        })
        .map(|s| s.as_str())
        .collect();

    if !available_metrics.is_empty() {
        debug!("Available resource metrics: {:?}", available_metrics);
    }

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

    #[test]
    fn test_parse_uptime_metric() {
        let text = r#"
cardano_node_metrics_upTime_ns 86400000000000
"#;
        let metrics = parse_prometheus_metrics(text);
        // 86400 seconds = 1 day
        assert_eq!(metrics.uptime_seconds, Some(86400.0));
    }

    #[test]
    fn test_parse_kes_metrics() {
        let text = r#"
cardano_node_metrics_currentKESPeriod_int 350
cardano_node_metrics_remainingKESPeriods_int 42
cardano_node_metrics_operationalCertificateExpiryKESPeriod_int 62
"#;
        let metrics = parse_prometheus_metrics(text);
        assert_eq!(metrics.kes_period, Some(350));
        assert_eq!(metrics.kes_remaining, Some(42));
        assert_eq!(metrics.kes_periods_per_cert, Some(62));
    }

    #[test]
    fn test_parse_p2p_metrics() {
        let text = r#"
cardano_node_metrics_p2p_enabled_int 1
cardano_node_metrics_p2p_coldPeersCount_int 5
cardano_node_metrics_p2p_warmPeersCount_int 15
cardano_node_metrics_p2p_hotPeersCount_int 12
cardano_node_metrics_connectionManager_incomingConns 10
cardano_node_metrics_connectionManager_outgoingConns 8
cardano_node_metrics_connectionManager_duplexConns 20
cardano_node_metrics_connectionManager_unidirectionalConns 8
"#;
        let metrics = parse_prometheus_metrics(text);
        assert_eq!(metrics.p2p.enabled, Some(true));
        assert_eq!(metrics.p2p.cold_peers, Some(5));
        assert_eq!(metrics.p2p.warm_peers, Some(15));
        assert_eq!(metrics.p2p.hot_peers, Some(12));
        assert_eq!(metrics.incoming_connections, Some(10));
        assert_eq!(metrics.outgoing_connections, Some(8));
        assert_eq!(metrics.full_duplex_connections, Some(20));
        assert_eq!(metrics.unidirectional_connections, Some(8));
    }
}
