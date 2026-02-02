# Prometheus Metrics Guide

## Overview

sview requires your Cardano node to expose Prometheus metrics. Not all metrics are available on all nodes - some depend on your node configuration, version, and role.

## Required Metrics for Core Features

### Basic Chain Metrics (Always Required)
```
cardano_node_metrics_blockNum_int             # Block height
cardano_node_metrics_slotNum_int              # Current slot number
cardano_node_metrics_epoch_int                # Current epoch
cardano_node_metrics_slotInEpoch_int          # Slot within epoch
cardano_node_metrics_connectedPeers_int       # Connected peer count
```

## Optional Metrics (May Not Be Present)

### Resource Metrics
These require special Prometheus configuration on the node:

```
cardano_node_metrics_RTS_gcLiveBytes_int      # Memory usage (WORKS)
cardano_node_metrics_RTS_cpuNs_int            # CPU time (OPTIONAL)
cardano_node_metrics_upTime_ns                # Node uptime (OPTIONAL)
```

### Mempool Metrics
Often not exposed by default:

```
cardano_node_metrics_txsInMempool_int         # Txs in mempool
cardano_node_metrics_mempoolBytes_int         # Mempool size in bytes
```

### KES Metrics
Block producers only:

```
cardano_node_metrics_currentKESPeriod_int     # Current KES period
cardano_node_metrics_remainingKESPeriods_int  # KES periods remaining
cardano_node_metrics_operationalCertificateExpiryKESPeriod_int
```

### P2P Metrics
Available on nodes with P2P enabled:

```
cardano_node_metrics_p2p_enabled_int
cardano_node_metrics_p2p_incomingConns_int
cardano_node_metrics_p2p_outgoingConns_int
cardano_node_metrics_p2p_coldPeersCount_int
cardano_node_metrics_p2p_warmPeersCount_int
cardano_node_metrics_p2p_hotPeersCount_int
cardano_node_metrics_p2p_unidirectionalPeersCount_int
cardano_node_metrics_p2p_bidirectionalPeersCount_int
cardano_node_metrics_p2p_fullDuplexPeersCount_int
```

## Debugging Missing Metrics

### Check What Metrics Your Node Exports

1. **Direct curl to Prometheus endpoint:**

```bash
curl http://localhost:12798/metrics | grep cardano_node_metrics | head -20
```

Replace `localhost` and `12798` with your node's host and Prometheus port.

2. **Filter for specific metrics:**

```bash
# Check for resource metrics
curl http://localhost:12798/metrics | grep -E "RTS|upTime|Mempool"

# Check for P2P metrics
curl http://localhost:12798/metrics | grep "p2p"

# Check for KES metrics
curl http://localhost:12798/metrics | grep "KES"
```

### Enable Debug Logging

Run sview with debug logging to see which metrics are being found:

```bash
RUST_LOG=debug sview
```

Look for log messages like:
```
Found metric: cardano_node_metrics_RTS_gcLiveBytes_int = 8000000000
Available resource metrics: ["cardano_node_metrics_RTS_gcLiveBytes_int"]
```

## Why Metrics Might Be Missing

### 1. Node Configuration

Some metrics require special configuration in your node config.json:

- **RTS metrics** (CPU, memory): Usually enabled by default
- **Mempool metrics**: May need `enablePrometheus` or similar setting
- **Detailed logging**: Some builds don't include all metrics

### 2. Node Version

Different Cardano node versions expose different metrics. Upgrade your node if metrics are missing.

### 3. Node Type

- **Relay nodes**: Limited metrics (no KES)
- **Block producers**: Full metrics including KES
- **New nodes**: May not have full P2P metrics yet

## Fixing Missing Resource Metrics

If uptime, CPU, or mempool metrics aren't showing:

### 1. Check if node exposes them:
```bash
curl http://localhost:12798/metrics | grep "upTime\|cpu\|Mempool"
```

### 2. Verify node configuration (node config.json)

Your Cardano node's `config.json` should have Prometheus enabled:
```json
{
  "hasPrometheus": ["127.0.0.1", 12798],
  "TurnOnLogging": true,
  "TurnOnLogMetrics": true,
  "TracingVerbosity": "NormalVerbosity"
}
```

**Key settings:**
- `"hasPrometheus": ["<bind_address>", <port>]` - Enables Prometheus metrics (default: 127.0.0.1:12798)
- `"TurnOnLogMetrics": true` - Enables metric collection
- `"TracingVerbosity": "NormalVerbosity"` - Sets logging detail level

### 3. Restart node
```bash
systemctl restart cardano-node
```

### 4. Verify metrics are now exposed
```bash
curl http://localhost:12798/metrics | grep -E "upTime|RTS_cpu|Mempool"
```

### Reference Configuration

For a complete example, see the official Cardano node configuration:
https://book.world.dev.cardano.org/environments/mainnet/config.json

## Expected Behavior

### Memory Used
✅ **Should always work** - This is a standard RTS metric

### Uptime, CPU Time, Mempool
⚠️ **May not appear** - Depends on node version and configuration

### KES
✅ **Block producers only** - Won't appear on relay nodes

### P2P Stats
⏳ **Newer nodes with P2P** - Not available on older versions

## Known Issues

### Metric Names Vary by Version
sview supports multiple naming conventions to handle different versions:
- `cardano_node_metrics_RTS_gcLiveBytes_int`
- `cardano_node_metrics_RTS_GCLiveBytes_int` 
- etc.

### Alternative Names

If you see metrics with slightly different names:
```bash
# Example: your node uses different naming
curl http://localhost:12798/metrics | grep memory
# Result: my_custom_memory_metric 1024000
```

Let us know and we can add support for your node's metric naming!

## Getting Help

To report missing metrics or get help:

1. Run: `RUST_LOG=debug sview 2>&1 | tee debug.log`
2. Save output from: `curl http://localhost:12798/metrics`
3. Include both in your issue report

## Summary

| Metric | Status | Depends On |
|--------|--------|-----------|
| Block Height | ✅ Always | Node running |
| Peer Count | ✅ Always | Node running |
| Memory Used | ✅ Usually | Node version |
| Uptime | ⚠️ Sometimes | Node config |
| CPU Time | ⚠️ Sometimes | Node version |
| Mempool TXs | ⚠️ Sometimes | Node config |
| KES | ✅ Block Producer | Role: BP |
| P2P Stats | ⏳ New nodes | Node version + P2P enabled |
