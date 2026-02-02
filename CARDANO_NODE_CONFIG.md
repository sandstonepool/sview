# Cardano Node Configuration for sview

## Overview

sview requires your Cardano node to expose Prometheus metrics. This guide shows how to properly configure your node.

## Official Configuration

The official Cardano node configuration for mainnet is available at:
https://book.world.dev.cardano.org/environments/mainnet/config.json

## Required Settings

### Prometheus Metrics

For sview to work, you **must** enable Prometheus in your node's `config.json`:

```json
{
  "hasPrometheus": ["127.0.0.1", 12798]
}
```

**Parameters:**
- First element: IP address to bind to (use `"127.0.0.1"` for local-only, or your server IP for remote access)
- Second element: Port number (default: 12798)

### Logging and Metrics

Recommended logging settings to ensure metrics are collected:

```json
{
  "TurnOnLogging": true,
  "TurnOnLogMetrics": true,
  "TracingVerbosity": "NormalVerbosity",
  "TurnOnLogMetrics": true
}
```

## Complete Example

Here's a minimal `config.json` snippet for a relay node:

```json
{
  "Protocol": "Cardano",
  "RequiresNetworkMagic": "RequiresNoMagic",
  "ByronGenesisFile": "byron-genesis.json",
  "ShelleyGenesisFile": "shelley-genesis.json",
  "AlonzoGenesisFile": "alonzo-genesis.json",
  "ConwayGenesisFile": "conway-genesis.json",
  
  "NodeToNodeProtocols": ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"],
  "NodeToClientProtocols": ["LocalStateQuery", "LocalTxSubmission", "LocalTxMonitoring"],
  
  "PeerSharing": true,
  "EnableP2P": true,
  
  "hasPrometheus": ["127.0.0.1", 12798],
  "hasEKG": 12788,
  
  "TurnOnLogging": true,
  "TurnOnLogMetrics": true,
  "TracingVerbosity": "NormalVerbosity"
}
```

## Network Binding

### Local-only (Development/Testing)
```json
"hasPrometheus": ["127.0.0.1", 12798]
```
- Only accessible from the same machine
- Safe for private networks

### Network-accessible (Production)
```json
"hasPrometheus": ["0.0.0.0", 12798]
```
- ⚠️ Accessible from any machine
- Use with firewall restrictions!

## Block Producer Configuration

For block producers, ensure you have the same settings plus KES metrics:

```json
{
  "hasPrometheus": ["127.0.0.1", 12798],
  "TurnOnLogging": true,
  "TurnOnLogMetrics": true,
  "ForgeTracingVerbosity": "MaximalVerbosity"
}
```

This enables detailed block forging metrics including KES periods.

## Verification

After restarting your node, verify Prometheus is working:

```bash
# Check if metrics endpoint is accessible
curl http://localhost:12798/metrics | head -20

# Look for specific metrics
curl http://localhost:12798/metrics | grep "cardano_node_metrics"

# Count total metrics
curl http://localhost:12798/metrics | grep -v "^#" | wc -l
```

You should see output like:
```
cardano_node_metrics_blockNum_int 12987530
cardano_node_metrics_slotNum_int 178453986
cardano_node_metrics_epoch_int 610
...
```

## Troubleshooting

### No metrics endpoint
**Problem:** `curl: (7) Failed to connect to localhost port 12798`

**Solution:** 
1. Verify `hasPrometheus` is in your `config.json`
2. Ensure node is running: `systemctl status cardano-node`
3. Restart node: `systemctl restart cardano-node`
4. Wait 10-15 seconds for startup

### Metrics endpoint unreachable from remote
**Problem:** Can curl locally but not from another machine

**Solution:**
1. Check firewall allows port 12798: `sudo ufw allow 12798`
2. Change binding from `127.0.0.1` to `0.0.0.0`
3. Verify with: `netstat -tlnp | grep 12798`

### Missing specific metrics
See [METRICS_GUIDE.md](METRICS_GUIDE.md) for information about which metrics are available and why some might be missing.

## Configuration Files Location

Typical paths for Cardano node configuration:

```
/opt/cardano/cnode/etc/config.json       # Guild operators
/home/cardano/config.json                # Manual installation
~/.cardano-node/config.json              # Development
```

## EKG Monitoring

The node also exposes monitoring via EKG on the port specified by `"hasEKG"`:

```json
{
  "hasEKG": 12788
}
```

This provides real-time resource monitoring (memory, CPU, threads) via HTTP, but sview uses Prometheus format which is preferred.

## Reference Files

- Official Mainnet Config: https://book.world.dev.cardano.org/environments/mainnet/config.json
- Official Testnet Config: https://book.world.dev.cardano.org/environments/testnet/config.json
- Cardano Documentation: https://developers.cardano.org

## Next Steps

Once your node is properly configured and metrics are exposed:

1. Run sview: `sview --prom-host localhost --prom-port 12798`
2. Or use config file: `~/.config/sview/config.toml`
3. See [README.md](README.md) for usage instructions
