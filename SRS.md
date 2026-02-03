# Software Requirements Specification (SRS)

## sview — Cardano Node Monitoring TUI

**Version:** 1.0  
**Last Updated:** 2026-02-02  
**Status:** Living Document

---

## 1. Introduction

### 1.1 Purpose
sview is a terminal-based monitoring tool for Cardano blockchain nodes. It provides real-time metrics visualization, health monitoring, and peer analysis through an intuitive TUI (Terminal User Interface).

### 1.2 Scope
This document defines the functional and non-functional requirements for sview. All development work should align with these specifications.

### 1.3 Target Users
- Cardano stake pool operators (SPOs)
- Node operators running relays or block producers
- Infrastructure teams managing Cardano deployments

### 1.4 Design Philosophy
- **Simplicity**: Single binary, no runtime dependencies
- **Performance**: Lightweight, minimal resource usage
- **Reliability**: Graceful handling of connection issues
- **Accessibility**: Works in standard terminals (80x24 minimum)

---

## 2. System Requirements

### 2.1 Supported Platforms
- Linux (primary target)
- macOS (secondary)
- Windows (via WSL)

### 2.2 Dependencies
- **Runtime**: None (statically linked binary)
- **Build**: Rust 1.75+ toolchain
- **Optional**: `ss` command for peer socket inspection

### 2.3 Cardano Node Requirements
- Prometheus metrics endpoint enabled
- Default port: 12798
- Required node configuration:
  ```json
  {
    "hasPrometheus": ["127.0.0.1", 12798],
    "TurnOnLogging": true,
    "TurnOnLogMetrics": true
  }
  ```

---

## 3. Functional Requirements

### 3.1 Core Metrics Display

#### 3.1.1 Chain Metrics
| Metric | Source | Display Format |
|--------|--------|----------------|
| Block Height | `blockNum_int` | Number with trend indicator (↑↓→) |
| Tip Age | Calculated from block time | "Xs ago" / "Xm Xs ago" |
| Slot Number | `slotNum_int` | Formatted number |
| Slot in Epoch | `slotInEpoch_int` | Formatted number |
| Epoch | `epoch_int` | Number |
| Chain Density | `density_real` | 4 decimal places |
| TX Processed | `txsProcessedNum_int/counter` | Formatted number |
| Forks | `forks_int/counter` | Formatted number |

#### 3.1.2 Network Metrics
| Metric | Source | Display Format |
|--------|--------|----------------|
| Connected Peers | Calculated from peer states | Number with trend indicator |
| Incoming Connections | `connectionManager_incomingConns` | Number |
| Outgoing Connections | `connectionManager_outgoingConns` | Number |
| Duplex Connections | `connectionManager_duplexConns` | Number |
| Unidirectional | `connectionManager_unidirectionalConns` | Number |
| Peer Distribution | Hot/Warm/Cold counts | Visual bar `[████▒▒░░░░]` |
| Block Delay | `blockfetchclient_blockdelay_s/real` | Milliseconds or seconds |
| Blocks Served | `served_block_count_int/counter` | Number |
| Blocks Late | `blockfetchclient_lateblocks` | Number, color-coded (0=green, 1-10=yellow, >10=red) |

#### 3.1.3 Block Propagation (CDF)
| Metric | Source | Display Format |
|--------|--------|----------------|
| Prop ≤1s | `blockdelay_cdfOne` | Percentage |
| Prop ≤3s | `blockdelay_cdfThree` | Percentage |
| Prop ≤5s | `blockdelay_cdfFive` | Percentage |

#### 3.1.4 Resource Metrics
| Metric | Source | Display Format |
|--------|--------|----------------|
| Uptime | Calculated from `nodeStartTime_int` | "Xd Xh Xm" |
| Memory Used | `RTS_gcLiveBytes_int` | GB/MB/KB |
| Memory Heap | `RTS_gcHeapBytes_int` | GB/MB/KB |
| GC Minor | `RTS_gcMinorNum_int` | Number |
| GC Major | `RTS_gcMajorNum_int` | Number |
| Mempool TXs | `txsInMempool_int` | Number |
| Mempool Size | `mempoolBytes_int` | Bytes formatted |

#### 3.1.5 Block Producer Metrics (when available)
| Metric | Source | Display Format |
|--------|--------|----------------|
| KES Remaining | `remainingKESPeriods_int` | "X (~Xd)" |
| Forging Status | `forging_enabled_int` | "Enabled"/"Disabled" |
| Blocks Forged | `Forge_adopted_int` | Number |
| Missed Slots | `slotsMissedNum_int` | Number, yellow if >0 |
| OpCert Status | Disk vs chain counter | "✓ X (valid)" or warning |

### 3.2 Health Indicators

#### 3.2.1 Color Coding
- 🟢 **Green (Healthy)**: Operating normally
- 🟡 **Yellow (Warning)**: Needs attention
- 🔴 **Red (Critical)**: Action required

#### 3.2.2 Health Thresholds
| Metric | Green | Yellow | Red |
|--------|-------|--------|-----|
| Sync Progress | ≥99.9% | ≥95% | <95% |
| Connected Peers | ≥5 | ≥2 | <2 |
| Memory Usage | <12GB | <14GB | ≥14GB |
| KES Remaining | ≥20 periods | ≥5 periods | <5 periods |
| Tip Age | <60s | <120s | ≥120s |

### 3.3 Progress Gauges

#### 3.3.1 Epoch Progress
- Visual progress bar showing current position in epoch
- Displays: Epoch number, percentage, time remaining
- Color changes as epoch progresses (green → yellow near end)

#### 3.3.2 Sync Progress
- Shows synchronization status
- Displays "Synced ✓" when ≥99.9%

#### 3.3.3 Memory Gauge
- Shows memory usage relative to heap
- Color-coded by health status

### 3.4 Multi-Node Support

#### 3.4.1 Requirements
- Support monitoring multiple nodes from single instance
- Tab-based navigation between nodes
- Per-node health indicators in tab bar
- Quick switching via number keys (1-9)

#### 3.4.2 Configuration
```toml
[global]
network = "mainnet"
refresh_interval_secs = 2

[[nodes]]
name = "Relay 1"
host = "10.0.0.1"
port = 12798
role = "relay"

[[nodes]]
name = "Block Producer"
host = "10.0.0.2"
port = 12798
role = "bp"
```

### 3.5 Peer List View

#### 3.5.1 Requirements
- Accessible via 'p' key
- Two display modes based on data availability:
  - **Full mode** (local): Individual peer details with IP, RTT, location
  - **Prometheus-only mode** (remote): Aggregate statistics from metrics

#### 3.5.2 Full Mode (Local)
When running on the same machine as the node, shows all connected peers with:
- IP address
- Port number
- Direction (IN/OUT)
- Location (city, country via GeoIP lookup)
- RTT latency (color-coded: green <50ms, yellow <100ms, red >100ms)
- Queue information (recv/send)
- Sorted by direction (incoming first), then RTT
- Summary showing total peers, direction breakdown, average RTT
- Peer detail view accessible by selecting peer and pressing Enter

#### 3.5.3 Prometheus-Only Mode (Remote)
When socket inspection fails (remote monitoring), shows aggregate data:
- Incoming/outgoing connection counts
- Duplex/unidirectional connection counts
- Peer state distribution with visual bars:
  - Cold peers (known but not promoted)
  - Warm peers (promoted but not active)
  - Hot peers (actively used)
- Info message explaining detailed view requires local access

#### 3.5.4 Socket Inspection
- Uses `ss` command (Linux) or `lsof` (macOS) to discover TCP connections
- Parses RTT from socket info (Linux only)
- Filters connections by node port
- Auto-detects mode based on socket inspection results

### 3.6 Theme System

#### 3.6.1 Available Themes
| Theme | Type | Description |
|-------|------|-------------|
| Dark Default | Dark | Cool blues and cyans |
| Dark Warm | Dark | Coral and peach tones |
| Dark Purple | Dark | Purple and magenta pastels |
| Dark Teal | Dark | Teal and mint greens |
| Light Default | Light | Soft blues on light background |
| Light Warm | Light | Peachy pastels |
| Light Cool | Light | Minty greens |

#### 3.6.2 Requirements
- Cycle through themes with 't' key
- WCAG AA accessible color contrast
- Theme persisted to config file
- Semantic color naming (primary, healthy, warning, critical, etc.)

### 3.7 Historical Graphs View

#### 3.7.1 Requirements
- Accessible via 'g' key
- Full-screen overlay showing sparkline graphs for key metrics
- Displays 5 metrics with historical data:
  - Block Height
  - Peers Connected
  - Memory Used (MB)
  - Mempool TXs
  - Sync Progress (%)
- Shows current value in each graph title
- Uses ring buffer history (~60 samples at 2s refresh = ~2 min)
- Footer shows sample count and time span
- Press Esc to close

### 3.8 Keyboard Navigation

| Key | Action | Context |
|-----|--------|---------|
| `q`, `Esc` | Quit / Close overlay | Global |
| `r` | Refresh metrics / peers | Global |
| `?` | Toggle help | Normal mode |
| `t` | Cycle theme | Normal mode |
| `p` | Toggle peer list | Normal mode |
| `g` | Toggle graphs view | Normal mode |
| `Tab` | Next node | Multi-node |
| `Shift+Tab` | Previous node | Multi-node |
| `1-9` | Select node by number | Multi-node |
| `←` `→` | Switch nodes | Multi-node |
| `↑` `↓`, `j` `k` | Navigate list | Peer list |
| `Enter` | View peer details | Peer list |

---

## 4. Non-Functional Requirements

### 4.1 Performance
- **Startup time**: <1 second
- **Memory footprint**: <50MB typical
- **CPU usage**: <1% idle, <5% during refresh
- **Refresh interval**: Configurable, default 2 seconds

### 4.2 Reliability
- Graceful handling of node connection failures
- Display last known values on temporary disconnect
- Clear error indication when node unreachable
- No crashes on malformed metrics data

### 4.3 Usability
- Minimum terminal size: 80x24
- Responsive layout adapting to terminal size
- Clear, readable text at default sizes
- Intuitive keyboard navigation

### 4.4 Compatibility
- Support cardano-node versions 8.x, 9.x, 10.x+
- Handle metric name variations between versions
- Support alternative node implementations (Dingo, Amaru)

---

## 5. Configuration

### 5.1 Configuration Sources (Priority Order)
1. CLI arguments (highest)
2. Environment variables
3. Config file (`~/.config/sview/config.toml`)

### 5.2 CLI Arguments
| Argument | Environment | Description | Default |
|----------|-------------|-------------|---------|
| `-n, --node-name` | `NODE_NAME` | Display name | "Cardano Node" |
| `--network` | `CARDANO_NETWORK` | Network name | "mainnet" |
| `--prom-host` | `PROM_HOST` | Prometheus host | "127.0.0.1" |
| `-p, --prom-port` | `PROM_PORT` | Prometheus port | 12798 |
| `--prom-timeout` | `PROM_TIMEOUT` | Timeout seconds | 3 |
| `-r, --refresh-interval` | `REFRESH_INTERVAL` | Refresh seconds | 2 |
| `--history-length` | `HISTORY_LENGTH` | History points | 60 |
| `--epoch-length` | `EPOCH_LENGTH` | Slots per epoch | 432000 |
| `-c, --config` | `SVIEW_CONFIG` | Config file path | `~/.config/sview/config.toml` |

### 5.3 Config File Format
```toml
[global]
network = "mainnet"
timeout_secs = 3
refresh_interval_secs = 2
history_length = 60
epoch_length = 432000
theme = "dark-default"

[[nodes]]
name = "My Node"
host = "127.0.0.1"
port = 12798
role = "relay"  # or "bp"
```

---

## 6. Data Storage

### 6.1 Persistent Storage
- Location: `~/.local/share/sview/`
- Format: Binary snapshots per node
- Retention: 30 days with hourly sampling

### 6.2 CSV Export
- Command: `sview --export <path>`
- Format: Standard CSV with headers
- Columns: timestamp, block_height, slot, peers, memory, etc.

---

## 7. Error Handling

### 7.1 Connection Errors
- Display "Connection Error" status
- Show last known values
- Automatic retry on next refresh interval

### 7.2 Parse Errors
- Skip malformed metric lines
- Log warnings (when RUST_LOG enabled)
- Continue with available metrics

### 7.3 System Errors
- Graceful terminal restore on crash
- Clear error messages for common issues

---

## 8. Future Considerations

### 8.1 Planned Features
- [x] IP geolocation for peers (integrated in peer list view)
- [x] Alert notifications (header display with ⚠ icon + file logging)
- [ ] Connection pooling optimization
- [ ] Async request cancellation

### 8.2 Out of Scope
- Remote node management (read-only monitoring)
- Historical charting beyond sparklines
- Web interface

---

## 9. Acceptance Criteria

### 9.1 Quality Gates
All releases must pass:
1. `cargo test --all` — All tests passing
2. `cargo fmt --check` — Code formatted
3. `cargo clippy --all-features -- -D warnings` — No warnings
4. `cargo build --release` — Clean compilation

### 9.2 Documentation Requirements
- README.md updated for new features
- Keyboard shortcuts documented
- Config options documented

---

## 10. Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-02-02 | Initial SRS based on v0.1.32 |
| 1.1 | 2026-02-02 | Updated for v0.1.42: GeoIP integration, alert display, peer detail view |
| 1.2 | 2026-02-03 | Updated for v0.1.55: Graceful degradation for peer view (Prometheus-only mode) |

---

*This is a living document. Update when requirements change.*
