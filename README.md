# sview

A TUI for monitoring Cardano nodes, written in Rust.

![License](https://img.shields.io/badge/license-Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
[![CI](https://github.com/sandstonepool/sview/actions/workflows/ci.yml/badge.svg)](https://github.com/sandstonepool/sview/actions/workflows/ci.yml)

## Overview

sview is a terminal-based monitoring tool for Cardano nodes. It provides real-time
metrics and status information by connecting to a node's Prometheus metrics endpoint.

Inspired by [nview](https://github.com/blinklabs-io/nview), sview is built from the
ground up in Rust using [ratatui](https://ratatui.rs) for a modern, responsive TUI
experience.

## Features

- 📊 Real-time node metrics display with sparkline history
- 🔍 Auto-detection of node type (cardano-node, Dingo, Amaru)
- 🚦 Color-coded health indicators (sync status, peer count, memory)
- 📅 Epoch progress bar with time remaining countdown
- ⚡ Lightweight and fast — single binary, no runtime dependencies
- 🎨 Clean, intuitive terminal interface
- 🔧 Flexible configuration via CLI arguments, environment variables, or config file
- 🖥️ **Multi-node monitoring** — watch all your relays and block producer from one terminal

## Installation

### From Releases

Download the latest binary for your platform from the [Releases](https://github.com/sandstonepool/sview/releases) page.

### From Source

```bash
cargo install --git https://github.com/sandstonepool/sview
```

### Build Locally

```bash
git clone https://github.com/sandstonepool/sview
cd sview
cargo build --release
```

## Usage

### Single Node (CLI)

```bash
# Default: connects to localhost:12798
sview

# Custom Prometheus endpoint
sview --prom-host 192.168.1.100 --prom-port 12798

# Set custom node name
sview --node-name "My Stake Pool"

# Using environment variables
PROM_HOST=192.168.1.100 NODE_NAME="My Stake Pool" sview
```

### Multi-Node (Config File)

Create a config file at `~/.config/sview/config.toml`:

```toml
[global]
network = "mainnet"
refresh_interval_secs = 2

[[node]]
name = "Relay 1"
host = "10.0.0.1"
port = 12798
role = "relay"

[[node]]
name = "Relay 2"
host = "10.0.0.2"
port = 12798
role = "relay"

[[node]]
name = "Block Producer"
host = "10.0.0.3"
port = 12798
role = "bp"
```

Then just run:

```bash
sview
```

Use `Tab` or number keys `1-9` to switch between nodes.

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q`, `Esc` | Quit |
| `r` | Force refresh metrics |
| `?` | Toggle help |
| `Tab` | Next node (multi-node mode) |
| `Shift+Tab` | Previous node (multi-node mode) |
| `1-9` | Select node by number (multi-node mode) |
| `←` `→` | Switch between nodes (multi-node mode) |

## Configuration

Configuration is loaded in the following order (later sources override earlier):

1. **Config file** (`~/.config/sview/config.toml` or `--config <path>`)
2. **Environment variables**
3. **CLI arguments**

If CLI arguments for `--prom-host` or `--prom-port` are provided, single-node mode is used regardless of config file.

### CLI Arguments

| Argument | Environment Variable | Description | Default |
|----------|---------------------|-------------|---------|
| `-n, --node-name` | `NODE_NAME` | Display name for the node | `Cardano Node` |
| `--network` | `CARDANO_NETWORK` | Network name (mainnet, preprod, preview) | `mainnet` |
| `--prom-host` | `PROM_HOST` | Prometheus metrics host | `127.0.0.1` |
| `-p, --prom-port` | `PROM_PORT` | Prometheus metrics port | `12798` |
| `--prom-timeout` | `PROM_TIMEOUT` | Request timeout in seconds | `3` |
| `-r, --refresh-interval` | `REFRESH_INTERVAL` | Refresh interval in seconds | `2` |
| `--history-length` | `HISTORY_LENGTH` | Data points to keep for sparklines | `60` |
| `--epoch-length` | `EPOCH_LENGTH` | Epoch length in slots | `432000` |
| `-c, --config` | `SVIEW_CONFIG` | Path to config file | `~/.config/sview/config.toml` |

### Config File Format

```toml
# Global settings (apply to all nodes unless overridden)
[global]
network = "mainnet"              # Default network for all nodes
timeout_secs = 3                 # Request timeout
refresh_interval_secs = 2        # How often to poll metrics
history_length = 60              # Sparkline data points
epoch_length = 432000            # Slots per epoch (432000 for mainnet)

# Node definitions (one [[node]] block per node)
[[node]]
name = "My Relay"                # Display name (required)
host = "127.0.0.1"               # Prometheus host (default: 127.0.0.1)
port = 12798                     # Prometheus port (default: 12798)
role = "relay"                   # "relay" or "bp" (block-producer)
network = "preprod"              # Override global network for this node
```

### Examples

```bash
# Monitor a remote node
sview --prom-host 10.0.0.5 -n "Relay 1"

# Slower refresh for low-bandwidth connections
sview --refresh-interval 5 --prom-timeout 10

# Monitor a testnet node (shorter epochs)
sview --network preprod --epoch-length 86400

# Use a custom config file
sview --config /path/to/my-config.toml

# Full example with all options
sview \
  --node-name "My Block Producer" \
  --network mainnet \
  --prom-host 192.168.1.100 \
  --prom-port 12798 \
  --refresh-interval 2 \
  --history-length 120
```

## Health Indicators

sview uses color-coded indicators to show node health at a glance:

| Color | Meaning |
|-------|---------|
| 🟢 Green | Healthy — operating normally |
| 🟡 Yellow | Warning — needs attention |
| 🔴 Red | Critical — action required |

**Health thresholds:**

- **Sync Progress**: Green ≥99.9%, Yellow ≥95%, Red <95%
- **Connected Peers**: Green ≥5, Yellow ≥2, Red <2
- **Memory Usage**: Green <12GB, Yellow <14GB, Red ≥14GB
- **KES Remaining**: Green ≥20 periods, Yellow ≥5, Red <5
- **Tip Age**: Green <60s, Yellow <120s, Red ≥120s

## Requirements

- A running Cardano node with Prometheus metrics enabled
- Terminal with Unicode support (minimum 80x24 recommended)

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

## Acknowledgments

- [nview](https://github.com/blinklabs-io/nview) — the original inspiration
- [ratatui](https://ratatui.rs) — the excellent Rust TUI library
