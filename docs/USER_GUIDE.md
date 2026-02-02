# User Guide

This guide covers all features of sview in detail.

## Dashboard Overview

The main dashboard is divided into several sections:

### Header Bar

```
┌─ sview — mainnet ─────────────────────────────────────────────────────────┐
│ NodeName [RELAY] ● ONLINE  │  Block: 10,500,000  E450  │  Peers: 25  │    │
│                            │  Health: ● Sync ● Peers ● Tip ● Mem         │
└───────────────────────────────────────────────────────────────────────────┘
```

- **Node Name**: Current node being monitored
- **Role Badge**: [RELAY] or [BLOCK PRODUCER]
- **Status**: ● ONLINE (green) or ○ OFFLINE (red)
- **Block**: Current block height with trend indicator (↑↓→)
- **Epoch**: Current epoch number (E450)
- **Peers**: Connected peer count with trend indicator
- **Health Indicators**: Quick status for Sync, Peers, Tip age, Memory

### Health Indicator Colors

| Color | Status | Meaning |
|-------|--------|---------|
| 🟢 Green | Good | Everything normal |
| 🟡 Yellow | Warning | Needs attention |
| 🔴 Red | Critical | Action required |

### Gauges

Three progress gauges show at-a-glance status:

1. **Epoch Progress**: Current position within the epoch with time remaining
2. **Sync Progress**: How synced the node is to the chain tip
3. **Memory Usage**: Current memory consumption vs heap size

### Metrics Columns

#### Chain Column
| Metric | Description |
|--------|-------------|
| Block Height | Current block number (with trend ↑↓→) |
| Tip Age | Time since last block was received |
| Slot | Current slot number |
| Slot in Epoch | Position within current epoch |
| Density | Chain density (blocks/slots ratio) |
| TX Processed | Total transactions processed |
| Forks | Number of chain forks encountered |
| KES Remaining | KES periods left (block producers only) |
| OpCert | Operational certificate validation status |

#### Network Column
| Metric | Description |
|--------|-------------|
| Connected | Total connected peers (with trend) |
| Incoming | Peers that connected to us |
| Outgoing | Peers we connected to |
| Duplex | Full-duplex (bidirectional) connections |
| Peer Dist | Distribution bar [████▒▒░░░░] H:5 W:3 C:10 |
| Block Delay | Average block propagation delay |
| Prop ≤1s | % of blocks received within 1 second |
| Prop ≤3s | % of blocks received within 3 seconds |
| Prop ≤5s | % of blocks received within 5 seconds |

#### Resources Column
| Metric | Description |
|--------|-------------|
| Uptime | Time since node started |
| Memory Used | Current memory usage (GC live bytes) |
| Memory Heap | Total heap size |
| GC Minor | Minor garbage collection count |
| GC Major | Major garbage collection count |
| Mempool TXs | Transactions in mempool |
| Mempool Size | Mempool size in bytes |

### Footer

```
 q quit  r refresh  p peers  t theme  ? help │ Tab next  1-9 select │ Updated 2s ago │ Dark Default
```

- Keyboard shortcuts
- Multi-node navigation hints (when applicable)
- Last update time
- Current theme name

## Multi-Node Monitoring

When configured with multiple nodes, a tab bar appears:

```
┌─ Nodes ───────────────────────────────────────────────────────────────────┐
│ ● Relay 1 [1] │ ● Relay 2 [2] │ ○ Block Producer BP [3]                   │
└───────────────────────────────────────────────────────────────────────────┘
```

- Green dot (●) = node online
- Gray dot (○) = node offline
- Number in brackets = quick-select key

### Switching Nodes

| Key | Action |
|-----|--------|
| `Tab` | Next node |
| `Shift+Tab` | Previous node |
| `←` `→` | Switch nodes |
| `1`-`9` | Select node by number |

## Peer List View

Press `p` to open the detailed peer list:

```
┌─ Peer Connections — 45 total (IN: 20 OUT: 25) — Avg RTT: 45.2ms [1-20/45] ─┐
│   DIR  IP ADDRESS       PORT   LOCATION          RTT       QUEUE           │
│ ▶ IN   203.0.113.50     3001   Sydney, AU        12.5ms    0               │
│   IN   198.51.100.25    3001   Tokyo, JP         85.3ms    0               │
│   OUT  192.0.2.100      3001   London, GB        120.0ms   R:0 S:128       │
│   OUT  192.0.2.101      3001   New York, US      95.5ms    0               │
│   ...                                                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│        [↑↓] select | [Enter] details | [p/Esc] close | [r] refresh         │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Peer List Columns

| Column | Description |
|--------|-------------|
| DIR | Direction: IN (incoming) or OUT (outgoing) |
| IP ADDRESS | Peer's IP address |
| PORT | Peer's port number |
| LOCATION | Geographic location (city, country) |
| RTT | Round-trip time latency |
| QUEUE | Receive/Send buffer status |

### RTT Color Coding

| Color | Latency | Quality |
|-------|---------|---------|
| Green | < 50ms | Excellent |
| Yellow | 50-100ms | Good |
| Red | > 100ms | Poor |

### Peer Detail View

Press `Enter` on a selected peer to see full details:

- IP Address and Port
- Geographic Location
- Connection Direction
- RTT Latency with quality assessment
- Connection State
- Buffer Queue sizes

## Theme System

sview includes 7 color themes optimized for readability:

### Dark Themes (for dark terminals)
- **Dark Default**: Cool blues and cyans
- **Dark Warm**: Coral and peach tones
- **Dark Purple**: Purple and magenta accents
- **Dark Teal**: Teal and mint greens

### Light Themes (for light terminals)
- **Light Default**: Deep blues on light background
- **Light Warm**: Warm earth tones
- **Light Cool**: Cool mint and teal

Press `t` to cycle through themes. Your preference is saved to the config file.

## Alert System

sview monitors for issues and displays alerts in the header:

```
│ ... Health: ● Sync ● Peers ● Tip ● Mem │  ⚠ KES Expiring Soon              │
```

### Alert Types

| Alert | Trigger | Severity |
|-------|---------|----------|
| KES Expiry | < 50 periods remaining | Critical (< 10), Warning (< 50) |
| Peer Drop | Peers dropped by > 30% | Warning |
| Sync Degradation | Sync progress decreased | Warning |
| Block Stall | No new blocks for > 5 minutes | Critical |

### Alert Logs

Alerts are logged to: `~/.local/share/sview/alerts/{node-name}.log`

## Help Overlay

Press `?` to see the help overlay with all keyboard shortcuts and health indicator explanations.

## Tips & Best Practices

### For Stake Pool Operators

1. **Monitor both relays and BP**: Configure all nodes in your pool
2. **Watch KES expiry**: Keep KES periods > 50 to avoid missed blocks
3. **Track peer quality**: Low RTT peers improve block propagation
4. **Check block propagation CDFs**: Aim for > 95% at 5 seconds

### For Relay Operators

1. **Maximize peer diversity**: Good geographic distribution
2. **Monitor incoming/outgoing balance**: Healthy relays have both
3. **Watch memory usage**: GC pressure indicates heap issues

### Performance Tips

1. **Use SSH tunneling** for remote monitoring (more secure than exposing ports)
2. **Set appropriate refresh interval**: 2-5 seconds is usually sufficient
3. **Use config file** for persistent settings across sessions

## Keyboard Quick Reference

| Key | Action |
|-----|--------|
| `q`, `Esc` | Quit / Close overlay |
| `r` | Refresh metrics |
| `p` | Toggle peer list |
| `t` | Cycle theme |
| `?` | Toggle help |
| `Tab` | Next node |
| `Shift+Tab` | Previous node |
| `1`-`9` | Select node |
| `↑`/`↓` | Navigate peer list |
| `Enter` | View peer details |

See [Keyboard Shortcuts](KEYBOARD_SHORTCUTS.md) for the complete reference.
