# Configuration Guide

sview can be configured via command-line arguments or a configuration file. For monitoring multiple nodes, a config file is recommended.

## Command-Line Options

```bash
sview [OPTIONS]

Options:
  -H, --host <HOST>      Prometheus metrics host [default: 127.0.0.1]
  -p, --port <PORT>      Prometheus metrics port [default: 12798]
  -c, --config <FILE>    Path to config file
  -n, --network <NET>    Network name (mainnet, preprod, preview) [default: mainnet]
      --export <FILE>    Export metrics to CSV and exit
  -h, --help             Print help
  -V, --version          Print version
```

## Configuration File

The config file uses TOML format. Default location: `~/.config/sview/config.toml`

### Basic Single-Node Config

```toml
[global]
network = "mainnet"

[[nodes]]
name = "My Relay"
host = "127.0.0.1"
port = 12798
```

### Multi-Node Config

```toml
[global]
network = "mainnet"
timeout_secs = 3
refresh_interval_secs = 2

[[nodes]]
name = "Relay 1"
host = "10.0.0.1"
port = 12798
role = "relay"

[[nodes]]
name = "Relay 2"
host = "10.0.0.2"
port = 12798
role = "relay"

[[nodes]]
name = "Block Producer"
host = "10.0.0.3"
port = 12798
role = "bp"
```

### Full Configuration Reference

```toml
[global]
# Network name (displayed in header)
network = "mainnet"

# HTTP timeout for metrics requests (seconds)
timeout_secs = 3

# How often to refresh metrics (seconds)
refresh_interval_secs = 2

# Number of historical data points to keep
history_length = 60

# Epoch length in slots (mainnet = 432000)
epoch_length = 432000

# Color theme (see THEMES section below)
theme = "dark-default"

[[nodes]]
# Display name for this node
name = "My Node"

# Prometheus metrics endpoint
host = "127.0.0.1"
port = 12798

# Node role: "relay" or "bp" (block-producer)
role = "relay"

# Override network for this specific node (optional)
network = "mainnet"
```

## Node Roles

Setting the correct node role helps sview display relevant information:

| Role | Value | Description |
|------|-------|-------------|
| Relay | `relay` | Standard relay node (default) |
| Block Producer | `bp` | Block producer with KES keys |

Block producers show additional metrics:
- KES period and remaining periods
- OpCert validation status
- Forging statistics (blocks adopted, missed slots)

## Theme Configuration

Available themes:

| Theme Name | Type | Description |
|------------|------|-------------|
| `dark-default` | Dark | Cool blues and cyans (default) |
| `dark-warm` | Dark | Coral and peach tones |
| `dark-purple` | Dark | Purple and magenta |
| `dark-teal` | Dark | Teal and mint greens |
| `light-default` | Light | Soft blues on light background |
| `light-warm` | Light | Peachy pastels |
| `light-cool` | Light | Minty greens |

Set in config:
```toml
[global]
theme = "dark-purple"
```

Or cycle through themes at runtime with the `t` key.

## Cardano Node Configuration

Your cardano-node must expose Prometheus metrics. Required node config:

```json
{
  "hasPrometheus": ["0.0.0.0", 12798],
  "TurnOnLogging": true,
  "TurnOnLogMetrics": true,
  "EnableP2P": true
}
```

### Security Note

If exposing metrics on `0.0.0.0` (all interfaces), ensure:
- Firewall restricts access to trusted IPs only
- Use VPN or private network for remote monitoring
- Never expose metrics port to the public internet

Recommended: Bind to `127.0.0.1` and use SSH tunneling for remote access:

```bash
# On your local machine
ssh -L 12798:localhost:12798 user@your-node-server

# Then run sview locally
sview --host 127.0.0.1 --port 12798
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SVIEW_CONFIG` | Path to config file |
| `RUST_LOG` | Logging level (error, warn, info, debug, trace) |

Example:
```bash
RUST_LOG=debug sview 2>&1 | tee sview.log
```

## Data Directories

| Path | Purpose |
|------|---------|
| `~/.config/sview/` | Configuration files |
| `~/.local/share/sview/` | Persistent data (history, alerts) |
| `~/.local/share/sview/alerts/` | Alert log files (per node) |

## Next Steps

- [User Guide](USER_GUIDE.md) - Learn all features
- [Keyboard Shortcuts](KEYBOARD_SHORTCUTS.md) - Quick reference
