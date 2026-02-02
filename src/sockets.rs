//! Socket inspection for peer discovery
//!
//! Uses system tools (ss) to discover connected peers and their connection details.

use std::process::Command;
use tracing::{debug, warn};

/// Information about a connected peer from socket inspection
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PeerConnection {
    /// Peer IP address
    pub ip: String,
    /// Peer port
    pub port: u16,
    /// Local port (our side)
    pub local_port: u16,
    /// Connection direction (true = incoming, false = outgoing)
    pub incoming: bool,
    /// Round-trip time in milliseconds (from ss)
    pub rtt_ms: Option<f64>,
    /// Connection state
    pub state: String,
    /// Receive queue bytes
    pub recv_q: u64,
    /// Send queue bytes
    pub send_q: u64,
}

impl PeerConnection {
    /// Get direction as string
    pub fn direction_str(&self) -> &'static str {
        if self.incoming {
            "IN"
        } else {
            "OUT"
        }
    }
}

/// Discover peer connections for a Cardano node
///
/// Uses `ss` command to inspect TCP connections on the node's port.
pub fn discover_peers(node_port: u16) -> Vec<PeerConnection> {
    let mut peers = Vec::new();

    // Use ss to get TCP connections with extended info
    // -t = TCP, -n = numeric, -i = internal TCP info (includes RTT)
    let output = match Command::new("ss")
        .args(["-tni", "state", "established"])
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            warn!("Failed to run ss command: {}", e);
            return peers;
        }
    };

    if !output.status.success() {
        warn!("ss command failed");
        return peers;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse ss output
    // Format:
    // Recv-Q Send-Q Local Address:Port Peer Address:Port Process
    //        cubic wscale:7,7 rto:204 rtt:1.875/0.625 ...

    let mut current_line: Option<(String, String, u64, u64)> = None; // (local, peer, recv_q, send_q)

    for line in stdout.lines() {
        let line = line.trim();

        // Skip header
        if line.starts_with("Recv-Q") || line.is_empty() {
            continue;
        }

        // Check if this is a connection line (starts with numbers for queues)
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 4 && parts[0].parse::<u64>().is_ok() {
            // This is a new connection line
            let recv_q = parts[0].parse().unwrap_or(0);
            let send_q = parts[1].parse().unwrap_or(0);
            let local = parts[2].to_string();
            let peer = parts[3].to_string();

            // Process previous connection if any
            if let Some((prev_local, prev_peer, prev_recv_q, prev_send_q)) = current_line.take() {
                if let Some(conn) = parse_connection(
                    &prev_local,
                    &prev_peer,
                    prev_recv_q,
                    prev_send_q,
                    None,
                    node_port,
                ) {
                    peers.push(conn);
                }
            }

            current_line = Some((local, peer, recv_q, send_q));
        } else if current_line.is_some() {
            // This is extended info line - look for RTT
            let rtt = parse_rtt(line);

            if let Some((local, peer, recv_q, send_q)) = current_line.take() {
                if let Some(conn) = parse_connection(&local, &peer, recv_q, send_q, rtt, node_port)
                {
                    peers.push(conn);
                }
            }
        }
    }

    // Process last connection
    if let Some((local, peer, recv_q, send_q)) = current_line {
        if let Some(conn) = parse_connection(&local, &peer, recv_q, send_q, None, node_port) {
            peers.push(conn);
        }
    }

    debug!("Discovered {} peer connections", peers.len());
    peers
}

/// Parse RTT from ss extended info line
fn parse_rtt(line: &str) -> Option<f64> {
    // Look for rtt:X.XXX/Y.YYY pattern
    for part in line.split_whitespace() {
        if part.starts_with("rtt:") {
            let rtt_str = part.trim_start_matches("rtt:");
            if let Some(slash_pos) = rtt_str.find('/') {
                if let Ok(rtt) = rtt_str[..slash_pos].parse::<f64>() {
                    return Some(rtt);
                }
            }
        }
    }
    None
}

/// Parse a connection from local/peer address strings
fn parse_connection(
    local: &str,
    peer: &str,
    recv_q: u64,
    send_q: u64,
    rtt: Option<f64>,
    node_port: u16,
) -> Option<PeerConnection> {
    // Parse addresses - format is either IP:port or [IPv6]:port
    let (local_ip, local_port) = parse_address(local)?;
    let (peer_ip, peer_port) = parse_address(peer)?;

    // Determine if this is related to our node port
    let is_node_local = local_port == node_port;
    let is_node_peer = peer_port == node_port;

    // Skip if neither side is our node port
    if !is_node_local && !is_node_peer {
        return None;
    }

    // Skip localhost connections
    if peer_ip == "127.0.0.1" || peer_ip == "::1" || local_ip == "127.0.0.1" || local_ip == "::1" {
        return None;
    }

    // Incoming = peer connected to our node port
    // Outgoing = we connected from our node to peer
    let incoming = is_node_local;

    Some(PeerConnection {
        ip: peer_ip,
        port: peer_port,
        local_port,
        incoming,
        rtt_ms: rtt,
        state: "ESTABLISHED".to_string(),
        recv_q,
        send_q,
    })
}

/// Parse IP:port or [IPv6]:port format
fn parse_address(addr: &str) -> Option<(String, u16)> {
    // Handle IPv6 [addr]:port format
    if addr.starts_with('[') {
        let bracket_end = addr.find(']')?;
        let ip = addr[1..bracket_end].to_string();
        let port_str = addr.get(bracket_end + 2..)?; // Skip ']:'
        let port = port_str.parse().ok()?;
        return Some((ip, port));
    }

    // Handle IPv4 addr:port format
    let colon_pos = addr.rfind(':')?;
    let ip = addr[..colon_pos].to_string();
    let port = addr[colon_pos + 1..].parse().ok()?;
    Some((ip, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address_ipv4() {
        let (ip, port) = parse_address("192.168.1.1:12798").unwrap();
        assert_eq!(ip, "192.168.1.1");
        assert_eq!(port, 12798);
    }

    #[test]
    fn test_parse_address_ipv6() {
        let (ip, port) = parse_address("[::1]:12798").unwrap();
        assert_eq!(ip, "::1");
        assert_eq!(port, 12798);
    }

    #[test]
    fn test_parse_rtt() {
        assert_eq!(parse_rtt("cubic wscale:7,7 rtt:1.875/0.625"), Some(1.875));
        assert_eq!(parse_rtt("cubic rtt:25.5/10.2 ato:40"), Some(25.5));
        assert_eq!(parse_rtt("no rtt here"), None);
    }
}
