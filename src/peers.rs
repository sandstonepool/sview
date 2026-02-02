//! Peer monitoring and statistics
//!
//! Tracks connected peers and their statistics based on Prometheus metrics.
//! Inspired by nview's peer monitoring approach but adapted for Prometheus-based metrics.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Peer connection direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum PeerDirection {
    /// Incoming connection
    Incoming,
    /// Outgoing connection
    Outgoing,
    /// Bidirectional/Duplex connection
    Duplex,
}

impl std::fmt::Display for PeerDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerDirection::Incoming => write!(f, "incoming"),
            PeerDirection::Outgoing => write!(f, "outgoing"),
            PeerDirection::Duplex => write!(f, "duplex"),
        }
    }
}

/// Peer state/temperature
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PeerState {
    /// Cold peer - known but not yet promoted
    Cold,
    /// Warm peer - promoted but not hot
    Warm,
    /// Hot peer - actively used
    Hot,
}

impl std::fmt::Display for PeerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerState::Cold => write!(f, "cold"),
            PeerState::Warm => write!(f, "warm"),
            PeerState::Hot => write!(f, "hot"),
        }
    }
}

/// Latency bucket for RTT classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LatencyBucket {
    /// RTT < 50ms
    VeryLow,
    /// RTT 50-100ms
    Low,
    /// RTT 100-200ms
    Medium,
    /// RTT > 200ms
    High,
    /// Unreachable/timeout
    Unreachable,
}

impl std::fmt::Display for LatencyBucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LatencyBucket::VeryLow => write!(f, "<50ms"),
            LatencyBucket::Low => write!(f, "50-100ms"),
            LatencyBucket::Medium => write!(f, "100-200ms"),
            LatencyBucket::High => write!(f, ">200ms"),
            LatencyBucket::Unreachable => write!(f, "unreachable"),
        }
    }
}

/// Individual peer information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Peer {
    /// Peer IP address (if available)
    pub ip: Option<String>,
    /// Peer port
    pub port: Option<u16>,
    /// Connection direction
    pub direction: PeerDirection,
    /// Peer state (cold/warm/hot)
    pub state: Option<PeerState>,
    /// Round-trip time in milliseconds
    pub rtt_ms: Option<u64>,
    /// Geolocation (country code)
    pub location: Option<String>,
    /// Last updated timestamp
    pub updated_at: u64,
}

impl Peer {
    /// Create a new peer with minimal info
    pub fn new(direction: PeerDirection) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            ip: None,
            port: None,
            direction,
            state: None,
            rtt_ms: None,
            location: None,
            updated_at: now,
        }
    }

    /// Get latency bucket for this peer's RTT
    pub fn latency_bucket(&self) -> LatencyBucket {
        match self.rtt_ms {
            Some(rtt) if rtt < 50 => LatencyBucket::VeryLow,
            Some(rtt) if rtt < 100 => LatencyBucket::Low,
            Some(rtt) if rtt < 200 => LatencyBucket::Medium,
            Some(rtt) if rtt < 99999 => LatencyBucket::High,
            _ => LatencyBucket::Unreachable,
        }
    }
}

/// Peer statistics aggregation
#[derive(Debug, Clone, Default)]
pub struct PeerStats {
    /// Total peers by state
    pub peers_by_state: HashMap<PeerState, u64>,
    /// Total peers by direction
    pub peers_by_direction: HashMap<PeerDirection, u64>,
    /// Peers by latency bucket
    pub peers_by_latency: HashMap<LatencyBucket, u64>,
    /// Average RTT in milliseconds
    pub avg_rtt_ms: u64,
    /// Sum of all RTTs (for calculating average)
    pub rtt_sum: u64,
    /// Count of reachable peers
    pub reachable_count: u64,
    /// Count of unreachable peers
    pub unreachable_count: u64,
    /// Percentage of peers in each bucket
    pub latency_percentages: HashMap<LatencyBucket, f32>,
}

impl PeerStats {
    /// Calculate statistics from a list of peers
    pub fn from_peers(peers: &[Peer]) -> Self {
        let mut stats = Self::default();

        for peer in peers {
            // Count by state
            if let Some(state) = peer.state {
                *stats.peers_by_state.entry(state).or_insert(0) += 1;
            }

            // Count by direction
            *stats.peers_by_direction.entry(peer.direction).or_insert(0) += 1;

            // Count by latency
            let bucket = peer.latency_bucket();
            *stats.peers_by_latency.entry(bucket).or_insert(0) += 1;

            // RTT aggregation
            if let Some(rtt) = peer.rtt_ms {
                if rtt < 99999 {
                    stats.rtt_sum += rtt;
                    stats.reachable_count += 1;
                } else {
                    stats.unreachable_count += 1;
                }
            }
        }

        // Calculate averages
        if stats.reachable_count > 0 {
            stats.avg_rtt_ms = stats.rtt_sum / stats.reachable_count;
        }

        // Calculate percentages
        let total_reachable = stats.reachable_count as f32;
        if total_reachable > 0.0 {
            for (bucket, count) in &stats.peers_by_latency {
                if *bucket != LatencyBucket::Unreachable {
                    stats.latency_percentages.insert(*bucket, (*count as f32) / total_reachable * 100.0);
                }
            }
            stats.latency_percentages.insert(
                LatencyBucket::Unreachable,
                (stats.unreachable_count as f32) / (stats.reachable_count as f32 + stats.unreachable_count as f32) * 100.0,
            );
        }

        stats
    }

    /// Get text summary of peer statistics
    #[allow(dead_code)]
    pub fn summary(&self) -> String {
        format!(
            "Peers: Hot={} Warm={} Cold={} | Dir: In={} Out={} Duplex={} | RTT: Avg={}ms | Reachable={} Unreachable={}",
            self.peers_by_state.get(&PeerState::Hot).unwrap_or(&0),
            self.peers_by_state.get(&PeerState::Warm).unwrap_or(&0),
            self.peers_by_state.get(&PeerState::Cold).unwrap_or(&0),
            self.peers_by_direction.get(&PeerDirection::Incoming).unwrap_or(&0),
            self.peers_by_direction.get(&PeerDirection::Outgoing).unwrap_or(&0),
            self.peers_by_direction.get(&PeerDirection::Duplex).unwrap_or(&0),
            self.avg_rtt_ms,
            self.reachable_count,
            self.unreachable_count,
        )
    }
}

/// Peer monitor for tracking peer state and statistics
#[derive(Debug, Clone, Default)]
pub struct PeerMonitor {
    /// List of tracked peers
    peers: Vec<Peer>,
    /// Cached statistics
    stats: PeerStats,
    /// Last update timestamp
    last_updated: u64,
}

impl PeerMonitor {
    /// Create a new peer monitor
    pub fn new() -> Self {
        Self::default()
    }

    /// Update peer list from metrics
    pub fn update_from_metrics(
        &mut self,
        hot_peers: Option<u64>,
        warm_peers: Option<u64>,
        cold_peers: Option<u64>,
        incoming_conns: Option<u64>,
        outgoing_conns: Option<u64>,
        duplex_conns: Option<u64>,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.peers.clear();

        // Create peers for each hot peer
        if let Some(count) = hot_peers {
            for _ in 0..count {
                let mut peer = Peer::new(PeerDirection::Duplex); // Default, will be refined
                peer.state = Some(PeerState::Hot);
                peer.updated_at = now;
                self.peers.push(peer);
            }
        }

        // Create peers for each warm peer
        if let Some(count) = warm_peers {
            for _ in 0..count {
                let mut peer = Peer::new(PeerDirection::Duplex);
                peer.state = Some(PeerState::Warm);
                peer.updated_at = now;
                self.peers.push(peer);
            }
        }

        // Create peers for each cold peer
        if let Some(count) = cold_peers {
            for _ in 0..count {
                let mut peer = Peer::new(PeerDirection::Incoming);
                peer.state = Some(PeerState::Cold);
                peer.updated_at = now;
                self.peers.push(peer);
            }
        }

        // Update direction counts (simplified: we don't have individual peer IPs from Prometheus)
        // In a full implementation with socket inspection, we'd merge these properly
        let _ = (incoming_conns, outgoing_conns, duplex_conns);

        // Recalculate statistics
        self.stats = PeerStats::from_peers(&self.peers);
        self.last_updated = now;
    }

    /// Get current statistics
    #[allow(dead_code)]
    pub fn stats(&self) -> &PeerStats {
        &self.stats
    }

    /// Get peer list
    #[allow(dead_code)]
    pub fn peers(&self) -> &[Peer] {
        &self.peers
    }

    /// Get peer count
    #[allow(dead_code)]
    pub fn count(&self) -> u64 {
        self.peers.len() as u64
    }

    /// Get last update timestamp
    #[allow(dead_code)]
    pub fn last_updated(&self) -> u64 {
        self.last_updated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_creation() {
        let peer = Peer::new(PeerDirection::Incoming);
        assert_eq!(peer.direction, PeerDirection::Incoming);
        assert_eq!(peer.state, None);
    }

    #[test]
    fn test_latency_bucket() {
        let mut peer = Peer::new(PeerDirection::Outgoing);
        peer.rtt_ms = Some(25);
        assert_eq!(peer.latency_bucket(), LatencyBucket::VeryLow);

        peer.rtt_ms = Some(75);
        assert_eq!(peer.latency_bucket(), LatencyBucket::Low);

        peer.rtt_ms = Some(150);
        assert_eq!(peer.latency_bucket(), LatencyBucket::Medium);

        peer.rtt_ms = Some(300);
        assert_eq!(peer.latency_bucket(), LatencyBucket::High);
    }

    #[test]
    fn test_peer_stats() {
        let mut peers = vec![];
        for i in 0..10 {
            let mut peer = Peer::new(if i < 5 {
                PeerDirection::Incoming
            } else {
                PeerDirection::Outgoing
            });
            peer.state = Some(if i < 3 {
                PeerState::Hot
            } else if i < 7 {
                PeerState::Warm
            } else {
                PeerState::Cold
            });
            peer.rtt_ms = Some((i * 20) as u64);
            peers.push(peer);
        }

        let stats = PeerStats::from_peers(&peers);
        assert_eq!(stats.peers_by_state.get(&PeerState::Hot).unwrap_or(&0), &3);
        assert_eq!(stats.peers_by_state.get(&PeerState::Warm).unwrap_or(&0), &4);
        assert_eq!(stats.peers_by_state.get(&PeerState::Cold).unwrap_or(&0), &3);
        assert_eq!(stats.reachable_count, 10);
    }

    #[test]
    fn test_peer_monitor() {
        let mut monitor = PeerMonitor::new();
        monitor.update_from_metrics(Some(5), Some(10), Some(20), Some(8), Some(12), Some(15));

        assert_eq!(monitor.count(), 35);
        assert_eq!(
            monitor.stats().peers_by_state.get(&PeerState::Hot).unwrap_or(&0),
            &5
        );
        assert_eq!(
            monitor.stats().peers_by_state.get(&PeerState::Warm).unwrap_or(&0),
            &10
        );
        assert_eq!(
            monitor.stats().peers_by_state.get(&PeerState::Cold).unwrap_or(&0),
            &20
        );
    }
}
