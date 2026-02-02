# Getting Started with sview

sview is a terminal-based monitoring tool for Cardano blockchain nodes. It provides real-time metrics visualization, health monitoring, and peer analysis through an intuitive TUI (Terminal User Interface).

## Quick Start

### 1. Installation

Download the latest release for your platform:

```bash
# Linux (x86_64)
curl -LO https://github.com/sandstonepool/sview/releases/latest/download/sview-linux-x86_64
chmod +x sview-linux-x86_64
sudo mv sview-linux-x86_64 /usr/local/bin/sview

# Or build from source
git clone https://github.com/sandstonepool/sview.git
cd sview
cargo build --release
sudo cp target/release/sview /usr/local/bin/
```

### 2. Configure Your Cardano Node

Ensure your cardano-node has Prometheus metrics enabled. Add to your node configuration:

```json
{
  "hasPrometheus": ["127.0.0.1", 12798],
  "TurnOnLogging": true,
  "TurnOnLogMetrics": true
}
```

### 3. Run sview

```bash
# Connect to local node (default: 127.0.0.1:12798)
sview

# Connect to remote node
sview --host 192.168.1.100 --port 12798

# Use a config file for multiple nodes
sview --config ~/.config/sview/config.toml
```

## What You'll See

When sview starts, you'll see a dashboard with three main sections:

```
┌─ sview — mainnet ─────────────────────────────────────────────────────────┐
│ NodeName [RELAY] ● ONLINE  │  Block: 10,500,000  E450  │  Peers: 25  │ ... │
├───────────────────┬────────────────────┬──────────────────────────────────┤
│    ┌─ Epoch ─┐    │    ┌─ Sync ─┐      │    ┌─ Memory ─┐                  │
│    │████████░│    │    │████████│      │    │██████░░░░│                  │
│    └─────────┘    │    └────────┘      │    └──────────┘                  │
│  ┌─ Chain ───┐    │  ┌─ Network ──┐    │  ┌─ Resources ─┐                 │
│  │Block  10M │    │  │Connected 25│    │  │Uptime 5d 3h│                  │
│  │Tip    12s │    │  │Incoming  12│    │  │Memory 4.2GB│                  │
│  │...        │    │  │...         │    │  │...         │                  │
│  └───────────┘    │  └────────────┘    │  └────────────┘                  │
└───────────────────┴────────────────────┴──────────────────────────────────┘
 q quit  r refresh  p peers  t theme  ? help
```

## Key Features

- **Real-time Metrics**: Block height, sync progress, peer connections, memory usage
- **Health Indicators**: Color-coded status (green=good, yellow=warning, red=critical)
- **Multi-Node Support**: Monitor multiple nodes from a single dashboard
- **Peer Analysis**: View connected peers with RTT latency and geolocation
- **Theme System**: 7 color themes for different preferences and lighting
- **Alert System**: Automatic alerts for KES expiry, peer drops, sync issues

## Next Steps

- [Configuration Guide](CONFIGURATION.md) - Set up multi-node monitoring
- [User Guide](USER_GUIDE.md) - Learn all features in detail
- [Keyboard Shortcuts](KEYBOARD_SHORTCUTS.md) - Quick reference
- [Troubleshooting](TROUBLESHOOTING.md) - Common issues and solutions
