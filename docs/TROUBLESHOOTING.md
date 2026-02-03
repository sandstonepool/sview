# Troubleshooting Guide

Common issues and their solutions.

## Connection Issues

### "Node Offline" or No Data

**Symptoms:**
- Status shows ○ OFFLINE
- All metrics show "—"
- Error in footer

**Solutions:**

1. **Check node is running:**
   ```bash
   systemctl status cardano-node
   # or
   ps aux | grep cardano-node
   ```

2. **Verify Prometheus endpoint:**
   ```bash
   curl http://127.0.0.1:12798/metrics
   ```
   Should return metrics text. If not:
   - Check `hasPrometheus` in node config
   - Verify the port is correct
   - Check firewall rules

3. **Check network connectivity:**
   ```bash
   # From sview machine to node
   nc -zv <node-ip> 12798
   ```

4. **Verify sview settings:**
   ```bash
   sview --prom-host <correct-ip> --prom-port <correct-port>
   ```

### Metrics Not Updating

**Symptoms:**
- Dashboard shows data but doesn't refresh
- "Updated Xs ago" keeps increasing

**Solutions:**

1. **Check node is syncing:**
   - If node is catching up, some metrics may be stale
   - Look at Sync gauge progress

2. **Increase timeout:**
   ```toml
   [global]
   timeout_secs = 5
   ```

3. **Check for network issues:**
   ```bash
   ping <node-ip>
   ```

## Missing Metrics

### TX Processed / Forks Show "—"

**Cause:** Different cardano-node versions use different metric names.

**Solution:** Update to latest sview version which handles multiple naming conventions.

**Debug:** Run with logging to see available metrics:
```bash
RUST_LOG=debug sview 2>&1 | grep -i "txs\|forks\|Unrecognized"
```

### KES Metrics Not Showing

**Cause:** Only block producers expose KES metrics.

**Solution:** 
- Set node role to `bp` in config
- Verify node has KES keys configured

```toml
[[nodes]]
name = "BP"
role = "bp"
```

### Peer Distribution Empty

**Cause:** P2P must be enabled on the node.

**Solution:** Ensure node config has:
```json
{
  "EnableP2P": true,
  "PeerSharing": true
}
```

### Memory Metrics Wrong

**Cause:** Multiple memory metrics exist; sview uses GC live bytes.

**Note:** `RTS_gcLiveBytes` shows actual used memory, not process RSS.

## Peer List Issues

### Peer List Shows "Prometheus" Mode

**Symptoms:**
- Peer list shows aggregate stats instead of individual peers
- Message says "Detailed peer info requires running sview on the node"

**Cause:** sview is running remotely and cannot inspect local sockets.

**Solutions:**
1. **Run sview on the same machine as the node** for full peer details
2. **Use SSH tunneling** to forward the Prometheus port, then run sview locally
3. **Accept the limitation** — Prometheus-only mode still shows connection counts and peer state distribution

### No Peers Found (Local Mode)

**Symptoms:**
- Peer list shows "No peer connections found"
- Press 'r' doesn't help

**Cause:** sview uses `ss` (Linux) or `lsof` (macOS) to discover peers.

**Solutions:**

1. **Verify ss/lsof is installed:**
   ```bash
   # Linux
   which ss
   # macOS
   which lsof
   ```

2. **Run sview on the same machine as the node:**
   - Peer discovery uses local socket inspection
   - Won't work when sview runs remotely

3. **Check node has connections:**
   ```bash
   # Linux
   ss -tni state established | head
   # macOS
   lsof -i TCP -n -P | grep cardano
   ```

### Wrong Peer Details

**Symptoms:**
- Selecting a peer shows different peer's details

**Solution:** Update to sview v0.1.42+ which fixes peer selection indexing.

### Peer Locations Not Showing

**Symptoms:**
- LOCATION column shows "—"

**Cause:** GeoIP lookup requires internet access.

**Solution:**
- Ensure sview can reach `ip-api.com`
- Private IPs (10.x, 192.168.x) won't have location data

## Display Issues

### Colors Look Wrong

**Symptoms:**
- Hard to read text
- Colors bleeding or missing

**Solutions:**

1. **Try different theme:**
   Press `t` to cycle through themes

2. **Check terminal color support:**
   ```bash
   echo $TERM
   # Should be xterm-256color or similar
   ```

3. **For light terminal backgrounds:**
   Use Light themes (Light Default, Light Warm, Light Cool)

### Gauge Labels Hard to Read

**Solution:** Update to sview v0.1.41+ which has improved gauge label contrast.

### Layout Broken

**Symptoms:**
- Columns misaligned
- Content cut off

**Solution:**
- Minimum terminal size: 80 columns × 24 rows
- Resize terminal larger for best experience
- Recommended: 120+ columns

## Performance Issues

### High CPU Usage

**Symptoms:**
- sview using significant CPU

**Solutions:**

1. **Increase refresh interval:**
   ```toml
   [global]
   refresh_interval_secs = 5
   ```

2. **Check node responsiveness:**
   - Slow metrics endpoint can cause issues

### Slow Peer List

**Symptoms:**
- Peer list takes long to open
- Refreshing peers is slow

**Cause:** GeoIP lookups for each peer.

**Note:** Locations are cached, subsequent views are faster.

## Configuration Issues

### Config File Not Found

**Solution:** Create config directory and file:
```bash
mkdir -p ~/.config/sview
cat > ~/.config/sview/config.toml << 'EOF'
[global]
network = "mainnet"

[[nodes]]
name = "My Node"
host = "127.0.0.1"
port = 12798
EOF
```

### Invalid TOML Syntax

**Symptoms:**
- sview fails to start
- Error about config parsing

**Common mistakes:**

```toml
# WRONG: [node] singular
[node]
name = "Test"

# CORRECT: [[nodes]] plural with double brackets
[[nodes]]
name = "Test"
```

### Theme Not Saving

**Solution:** Ensure config file is writable:
```bash
chmod 644 ~/.config/sview/config.toml
```

## Debug Mode

For detailed troubleshooting, run with debug logging:

```bash
RUST_LOG=debug sview 2>&1 | tee sview-debug.log
```

This logs:
- All HTTP requests
- Metric parsing details
- Unrecognized metrics
- Error details

## Getting Help

If issues persist:

1. **Check GitHub Issues:**
   https://github.com/sandstonepool/sview/issues

2. **Open a New Issue:**
   Include:
   - sview version (`sview --version`)
   - cardano-node version
   - Config file (redact sensitive info)
   - Debug log output
   - Terminal type and size

3. **Community Support:**
   Cardano SPO communities on Telegram/Discord
