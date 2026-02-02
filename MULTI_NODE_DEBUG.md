# Multi-Node Mode Debugging Guide

If multi-node support is not working as expected, follow this guide to diagnose the issue.

## Prerequisites

- Config file must exist at `~/.config/sview/config.toml`
- Config file must use `[[nodes]]` (plural) for node definitions, NOT `[[node]]`
- At least 2 nodes defined in the config file

## Quick Test

1. Create `~/.config/sview/config.toml` with this content:

```toml
[global]
network = "mainnet"
refresh_interval_secs = 2

[[nodes]]
name = "Relay 1"
host = "127.0.0.1"
port = 12798
role = "relay"

[[nodes]]
name = "Relay 2"
host = "127.0.0.1"
port = 12799
role = "relay"
```

2. Run sview:

```bash
sview
```

You should see:
- Node tabs at the top showing "Relay 1" and "Relay 2"
- Ability to switch nodes with Tab, Shift+Tab, arrow keys, or number keys (1, 2)
- Help text showing multi-node navigation options (press `?`)

## Common Issues

### Issue: Only see one node, not two

**Cause:** Config file is not being read, or nodes are not loading.

**Check:**
1. Verify config file exists: `ls -la ~/.config/sview/config.toml`
2. Verify syntax uses `[[nodes]]` NOT `[[node]]`
3. Try explicitly passing config: `sview --config ~/.config/sview/config.toml`

**Debug:**
- Add debug output to see if config is loading (check stderr for tracing messages)
- Use `RUST_LOG=debug sview` to enable debug logging

### Issue: Node tabs appear, but Tab key doesn't work

**Cause:** Event handling issue or wrong key code captured.

**Check:**
1. Try alternative navigation: Shift+Tab, arrow keys, number keys (1, 2, 3, etc.)
2. Verify you're actually in multi-node mode (should see node tabs at top)

**Debug:**
- The app should be receiving Tab keypresses and cycling nodes
- Try pressing `1` to select first node, `2` to select second node

### Issue: Can't start app with config file

**Cause:** Config file path or TOML parsing error.

**Check:**
1. Validate TOML syntax using an online validator
2. Ensure `[[nodes]]` blocks are properly formatted
3. Check for typos in field names (should be: `name`, `host`, `port`, `role`, `network`)

**Debug:**
```bash
# Try with explicit config path
sview --config ~/.config/sview/config.toml

# Try with single-node mode first to ensure app works
sview --prom-host 127.0.0.1 --prom-port 12798 -n "Test"
```

## Config File Format

Correct format:

```toml
[global]
network = "mainnet"
refresh_interval_secs = 2

[[nodes]]
name = "Node 1"
host = "10.0.0.1"
port = 12798
role = "relay"

[[nodes]]
name = "Node 2"
host = "10.0.0.2"
port = 12798
role = "bp"
```

**WRONG** (using singular `[[node]]`):
```toml
[[node]]  # ❌ WRONG - should be [[nodes]]
name = "Node 1"
```

## Keyboard Navigation

When in multi-node mode, use:

| Key | Action |
|-----|--------|
| `Tab` | Next node |
| `Shift+Tab` | Previous node |
| `←` | Previous node |
| `→` | Next node |
| `1-9` | Select specific node by number |

## Verify Node Loading

1. Look at the top of the screen - you should see node tabs
2. Press `?` to open help - multi-node section should appear if 2+ nodes detected
3. Each tab should show:
   - Green/yellow/red indicator (health status)
   - Node name
   - `[BP]` badge if block producer role
   - Number in brackets (e.g., `[1]`, `[2]`)

## Still Not Working?

1. Verify `~/.config/sview/` directory exists: `mkdir -p ~/.config/sview`
2. Use correct TOML syntax with `[[nodes]]` (plural)
3. Ensure at least 2 nodes are defined
4. Try running with debug logging: `RUST_LOG=debug sview`
5. Check if running in single-node mode by testing: `sview --prom-host 127.0.0.1 -n "Test"`

## Report Issue

If still not working, run with debug logging and save output:

```bash
RUST_LOG=debug sview 2>&1 | tee debug.log
```

Include `debug.log` and your `~/.config/sview/config.toml` when reporting the issue.
