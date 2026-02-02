# sview

A TUI for monitoring Cardano nodes, written in Rust.

![License](https://img.shields.io/badge/license-Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)

## Overview

sview is a terminal-based monitoring tool for Cardano nodes. It provides real-time
metrics and status information by connecting to a node's Prometheus metrics endpoint.

Inspired by [nview](https://github.com/blinklabs-io/nview), sview is built from the
ground up in Rust using [ratatui](https://ratatui.rs) for a modern, responsive TUI
experience.

## Features

- 📊 Real-time node metrics display
- 🔍 Auto-detection of node type (cardano-node, Dingo, Amaru)
- ⚡ Lightweight and fast — single binary, no runtime dependencies
- 🎨 Clean, intuitive terminal interface
- 🔧 12-factor configuration via environment variables

## Installation

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

```bash
# Default: connects to localhost:12798
sview

# Custom Prometheus endpoint
PROM_HOST=192.168.1.100 PROM_PORT=12798 sview

# Set custom node name
NODE_NAME="My Stake Pool" sview
```

## Configuration

Configuration is done via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `NODE_NAME` | Display name for the node | `Cardano Node` |
| `CARDANO_NETWORK` | Network name (mainnet, preprod, preview) | `mainnet` |
| `PROM_HOST` | Prometheus metrics host | `127.0.0.1` |
| `PROM_PORT` | Prometheus metrics port | `12798` |
| `PROM_TIMEOUT` | Request timeout in seconds | `3` |

## Requirements

- A running Cardano node with Prometheus metrics enabled
- Terminal with Unicode support (minimum 80x24 recommended)

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

## Acknowledgments

- [nview](https://github.com/blinklabs-io/nview) — the original inspiration
- [ratatui](https://ratatui.rs) — the excellent Rust TUI library
