# sview TUI Design

**Version:** v0.1.21 (Living Document)  
**Last Updated:** 2026-02-02  
**Maintainer:** Claw Daddy  

This document describes the complete Terminal User Interface (TUI) design for sview, a real-time Cardano node monitoring tool. It serves as the source of truth for the UI/UX and should be updated with all TUI changes.

---

## 📐 Design Philosophy

**Goals:**
- Display all available metrics at a glance
- Prioritize critical information for operational awareness
- Maintain consistent information hierarchy
- Support both relay and block producer monitoring
- Optimize for varied terminal sizes
- Color-coded health indicators for quick scanning

**Constraints:**
- Single terminal window (no external windows)
- Responsive to 80x24 minimum (practical: 120x40+)
- Real-time updates with 100ms poll interval
- Multi-node support with quick navigation
- Consistent across light and dark themes

---

## 🎨 Current Layout (v0.1.21+)

```
┌─────────────────────────────────────────────────────────────────────┐
│ [●] Cardano Node [BP] Connected │ Network: mainnet │ Node: cardano  │
│                                 │ Theme: Dark Default              │
├─────────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────── EPOCH PROGRESS ──────────┐ │
│ │ ███████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │ │
│ │ 78.2% — 2d 4h 23m remaining                                      │ │
│ └────────────────────────────────────────────────────────────────┘ │
│
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│  │  CHAIN METRICS   │  │ NETWORK & PEERS  │  │   RESOURCES      │
│  ├──────────────────┤  ├──────────────────┤  ├──────────────────┤
│  │ Block:  12987530 │  │ Incoming:     35 │  │ Uptime: 45d 3h  │
│  │ Tip Age:    12s  │  │ Outgoing:     40 │  │ Memory: 7.5 GB  │
│  │ Slot:  178453986 │  │ Duplex:        6 │  │ Heap:   12.1 GB │
│  │ Epoch: 610/86400 │  │ Unidirect:    34 │  │ GC Min:   89542 │
│  │ Sync:    99.98% ⬅  │ Block Delay: 45ms │  │ GC Maj:    1230 │
│  │ Density: 0.9994  │  │ Served:    892104 │  │ CPU:  2h 14m 3s │
│  │ TX Proc:   45123 │  │ Late:          12 │  │ Mempool TX:  234 │
│  │ Forks:        0  │  │                  │  │ Mempool:  1.2 MB │
│  │ Peers:       59  │  │  P2P BREAKDOWN   │  │ Blocks Adopted:  │
│  │ KES Rem:  324    │  │  ├─ Cold:   120  │  │ Adopted:    2841 │
│  │                  │  │  ├─ Warm:    22  │  │ Failed:        0 │
│  │                  │  │  ├─ Hot:     20  │  │                  │
│  │                  │  │  ├─ Duplex:   6  │  │                  │
│  │                  │  │  ├─ Bidirect: 12 │  │                  │
│  │                  │  │  └─ Unidirect:42 │  │                  │
│  │                  │  │                  │  │                  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘
│
│  ┌─────────────────────────────┬─────────────────────────────────┐
│  │    BLOCK HEIGHT SPARKLINE   │   MEMORY USAGE SPARKLINE       │
│  │ ▁▂▃▅▇█▇▅▃▂▁▂▃▅▇█▇▅▃▂▁▂▃▅▇ │ ▂▃▃▃▂▁▂▃▂▃▃▃▂▁▂▃▂▃▃▃▂▃▂▃▄▅█ │
│  │ Blocks (0.15/min)           │ Trend → (within safe range)    │
│  └─────────────────────────────┴─────────────────────────────────┘
│
├─────────────────────────────────────────────────────────────────────┤
│ [q]uit [r]efresh [t]heme [?]help [Tab]node [1-9]select│ 10.0.0.1:12798
└─────────────────────────────────────────────────────────────────────┘
```

---

## 📋 Layout Structure

### Header (3 lines)
**Content:** Node identification and status  
- Node name + role indicator ([BP] for block producer)
- Connection status (● connected, ○ disconnected)
- Network name (mainnet, testnet, etc.)
- Node type (cardano-node, Dingo, Amaru)
- Current theme name

**Colors:**
- Node name: text (white/black per theme)
- Status indicator: health color (green/yellow/red)
- Labels: muted text (gray)

---

### Epoch Progress (3 lines, full width)
**Content:** Epoch progress gauge with countdown  
- Full-width progress bar
- Percentage completed
- Time remaining in epoch
- Color changes with progress (green → yellow → orange as epoch end approaches)

**Significance:**
- Critical for operators managing validator schedule
- Shows when new epoch starts (trigger for KES key rotations)
- Helps plan maintenance windows
- Always at-a-glance visible

---

### Metrics Section (~11-14 lines)
**Three equal columns (33% / 33% / 34%)**

#### Left Column: Chain Metrics
Focus on blockchain state and consensus.

| Metric | Display | Health? | Range |
|--------|---------|---------|-------|
| Block Height | `12987530` | No | u64 |
| Tip Age | `12s ago` | ✅ | <60s=Good |
| Slot Number | `178453986` | No | u64 |
| Epoch/Slot | `610/86400` | No | u64/u64 |
| Sync Progress | `99.98%` | ✅ | 0-100% |
| Chain Density | `0.9994` | No | 0-1.0 |
| TX Processed | `45123` | No | u64 |
| Forks | `0` | No | u64 |
| Connected Peers | `59` | ✅ | Warn <2 |
| KES Remaining | `324` | ✅ | Crit <5 |

**Note:** Epoch progress visualization moved to full-width section at top (see Epoch Progress section)

---

#### Middle Column: Network & P2P Metrics
Focus on connectivity and peer distribution.

| Section | Metrics | Display |
|---------|---------|---------|
| **Connections** | Incoming | `35` |
| | Outgoing | `40` |
| | Duplex | `6` |
| | Unidirectional | `34` |
| | Prunable | `0` |
| **Block Fetch** | Block Delay | `45ms` |
| | Blocks Served | `892104` |
| | Blocks Late | `12` |
| **P2P Peers** | Cold (not connected) | `120` |
| | Warm (known) | `22` |
| | Hot (active) | `20` |
| | Duplex Peers | `6` |
| | Bidirectional | `12` |
| | Unidirectional | `42` |

---

#### Right Column: Resource & Forging Metrics
Focus on system health and block production.

| Metric | Display | Health? | Threshold |
|--------|---------|---------|-----------|
| Uptime | `45d 3h 12m` | No | Info |
| Memory Used | `7.5 GB` | ✅ | Warn >10GB |
| Memory Heap | `12.1 GB` | No | Info |
| GC Minor | `89542` | No | Info |
| GC Major | `1230` | ⚠️ | Warn >100/h |
| CPU Time | `2h 14m 3s` | No | Info |
| Mempool TXs | `234` | No | Info |
| Mempool Size | `1.2 MB` | No | Info |
| Blocks Adopted | `2841` | No | BP only |
| Blocks Failed | `0` | No | BP only |

---

### Memory Gauge (Top Right, 3 lines)
**Width:** 40% of terminal width (next to epoch progress)

**Content:**
- Horizontal progress gauge showing memory usage ratio
- Label shows current memory in MB/GB
- Color: Healthy (green) → Warning (yellow) → Critical (red)
- Calculation: Used / Total heap

**Significance:**
- Real-time memory pressure indicator
- Health coloring shows when approaching limits
- Side-by-side with epoch progress for quick visual scan

---

### Footer (3 lines)
**Content:** Keyboard shortcuts and endpoint

**Left side (shortcuts):**
```
[q]uit [r]efresh [t]heme [?]help [Tab]node [1-9]select
```

**Right side (endpoint):**
```
│ 10.0.0.1:12798
```

---

## 🎨 Color System

### Themes (7 available)

Users can cycle themes with `[t]` key. Current theme displayed in header.

**Dark Themes:**
1. **Dark Default** - Cool blues & greens (hacker aesthetic)
2. **Dark Warm** - Oranges & pinks (cozy)
3. **Dark Purple** - Purple-dominant (elegant)
4. **Dark Teal** - Teal & cyan (modern, accessible)

**Light Themes:**
5. **Light Default** - Blue pastels (professional)
6. **Light Warm** - Peachy pastels (gentle)
7. **Light Cool** - Minty pastels (fresh)

### Health Colors (Consistent across all themes)

All themes use semantic colors for status:
- **Green (Healthy):** Good state, no action needed
- **Yellow (Warning):** Attention required, monitor closely
- **Red (Critical):** Immediate action needed

Specific applications:
- **Tip Age:** Red if >120s, yellow if >60s, green otherwise
- **Sync Progress:** Red if <90%, yellow if <95%, green otherwise
- **Connected Peers:** Red if 0, yellow if <2, green otherwise
- **Memory Used:** Red if >90% heap, yellow if >75%, green otherwise
- **KES Remaining:** Red if <5 periods, yellow if <10, green otherwise

### UI Elements

- **Borders:** Theme border color (muted)
- **Text:** Theme text color (white/black per mode)
- **Muted text:** Theme text_muted color (gray)
- **Accents:** Theme primary/secondary/tertiary colors
- **Sparklines:** Theme sparkline color (usually primary)

---

## ⌨️ Keyboard Controls

| Key | Action | Context |
|-----|--------|---------|
| `q` | Quit | Always |
| `Esc` | Quit | Always |
| `r` | Force refresh metrics | Always |
| `?` | Toggle help popup | Always |
| `t` | Cycle color theme | Always |
| `Tab` | Next node | Multi-node mode |
| `Shift+Tab` | Previous node | Multi-node mode |
| `→` | Next node | Multi-node mode |
| `←` | Previous node | Multi-node mode |
| `1-9` | Select node by number | Multi-node mode |

**Help Popup** (`?`):
- Shows all keyboard shortcuts
- Lists health indicators
- Explains color meanings
- Press any key to close

---

## 📊 Metrics Reference

### Chain Metrics (Left Column)
- **Block Height:** Current blockchain tip (highest block number)
- **Tip Age:** Time since last block received (staleness indicator)
- **Slot Number:** Current slot in current epoch
- **Epoch/Slot:** Current epoch number and slot within epoch
- **Sync Progress:** % of chain synced (0-100%, 99.9%+ = synced)
- **Chain Density:** Theoretical max chain quality (0-1.0, higher = better)
- **TX Processed:** Total transactions processed since startup
- **Forks:** Number of blockchain forks encountered
- **Connected Peers:** Active P2P connections
- **KES Remaining:** KES periods until certificate expires (block producer only)

### Network Metrics (Middle Column)
- **Incoming Connections:** Inbound P2P connections
- **Outgoing Connections:** Outbound P2P connections
- **Duplex Connections:** Full-duplex peer connections
- **Unidirectional:** One-way peer connections
- **Block Delay:** Average time to receive new block
- **Blocks Served:** Total blocks shared with peers
- **Blocks Late:** Blocks received after deadline
- **Cold Peers:** Not-yet-connected peers (peer selection pool)
- **Warm Peers:** Known but not active peers
- **Hot Peers:** Actively syncing peers
- **Duplex Peers:** Full-duplex peer count
- **Bidirectional Peers:** Both-directions-open peers
- **Unidirectional Peers:** One-direction-only peers

### Resource Metrics (Right Column)
- **Uptime:** Node operational duration since startup
- **Memory Used:** GC live heap bytes (primary memory metric)
- **Memory Heap:** Total heap allocation
- **GC Minor:** Minor garbage collection count
- **GC Major:** Major garbage collection count
- **CPU Time:** Cumulative GC CPU time in ms
- **Mempool TXs:** Pending transactions in mempool
- **Mempool Size:** Pending bytes in mempool
- **Blocks Adopted:** Successfully forged blocks (block producer only)
- **Blocks Failed:** Failed forge attempts (block producer only)

---

## 📈 Information Hierarchy

**Critical (Top, Always Visible):**
- Node status (connected/disconnected)
- Block height (chain tip)
- Tip age (freshness)
- Sync progress (catching up?)
- Connected peers (isolated?)
- Memory usage (resource pressure)

**Important (Prominent):**
- Epoch progress
- Block delay
- Incoming/outgoing connections
- Uptime
- CPU usage

**Secondary (Still Visible):**
- Chain density, forks, transactions processed
- Peer classifications (cold/warm/hot)
- GC statistics
- Mempool metrics
- Forging stats (BP only)

**Trends (Bottom):**
- Block height sparkline (are blocks coming in?)
- Memory usage sparkline (is usage stable?)

---

## 🔧 Single vs Multi-Node Mode

### Single-Node Mode
- CLI or config file with single node
- Node tabs hidden (no need to switch)
- Full screen for single node metrics

### Multi-Node Mode
- Config file with multiple `[[nodes]]` sections
- Node tabs visible at top (shows all nodes at a glance)
- Quick switching with Tab, arrow keys, or number keys
- Each node gets its own metrics, history, and alerts

**Node Indicator:**
- `●` (filled circle) = connected
- `○` (empty circle) = disconnected
- Color = health status (green/yellow/red)
- Shortcut number shown `[1]` through `[9]`

---

## 🎯 Design Decisions

### Why 3-Column Layout?
- **Logical grouping:** Chain | Network | Resources
- **Equals space:** Each gets ~33% of width
- **Scanning:** Left-to-right matches operational priority
- **Scalability:** Fits 80+ column terminals
- **Multi-node:** Can swap nodes without layout change

### Why Sparklines at Bottom?
- **Secondary importance:** Trends, not absolute values
- **Consistent height:** Both sparklines exactly 5 lines
- **50/50 split:** Block and Memory are equally important
- **Space efficiency:** Takes minimal vertical space
- **Fast updates:** Easy to see changes as data streams in

### Why Health Coloring?
- **Quick scanning:** Red jumps out immediately
- **Universal:** Color means same thing across metrics
- **Accessibility:** Yellow/green distinction clear even for color blindness
- **Consistent:** Same green/yellow/red everywhere

### Why Semantic Metric Names?
- **Clarity:** "Tip Age" not "T_Age_Secs"
- **Context:** Full names in labels, abbreviations only in values
- **Consistency:** Same metric always called the same thing
- **Searchability:** Easy to find in documentation

---

## 📱 Terminal Size Support

**Minimum:**
- Width: 80 columns (though 120+ recommended)
- Height: 24 lines (though 40+ recommended)

**Typical:**
- Width: 120-160 columns
- Height: 40-50 lines

**Layout Adaptation:**
- Metrics always 3 columns (no stacking at narrow widths)
- Sparklines scale to available width (always 50/50 split)
- If terminal too narrow: Text truncates with `...`
- If terminal too short: May need scroll (not currently implemented)

---

## 🔮 Future Enhancements

### Phase 2 (v0.2.0)
- Active alerts display (currently logged only)
- Alert banner at top of metrics
- Per-node alert history

### Phase 3 (v0.2.1)
- Scrollable metric sections
- Collapsible advanced metrics
- Customizable column visibility
- Save/restore layout preferences

### Phase 4 (v0.2.2)
- Trend indicators (↑↓→ for metrics)
- Memory trend direction
- Peer count trending
- Block rate trending

### Phase 5+ (Future)
- Multiple page layouts
- Custom metric selection
- Export metrics to JSON
- Web dashboard alternative

---

## 🔄 Updates to This Document

This document should be updated whenever:
1. **Layout changes:** New sections, reorganization, sizing changes
2. **Metrics added/removed:** Update metrics reference section
3. **Color schemes change:** Update color system section
4. **Keyboard shortcuts added:** Update controls section
5. **Information hierarchy changes:** Update design philosophy
6. **Theme changes:** Update color system

**Update checklist:**
- [ ] Update version number
- [ ] Update "Last Updated" date
- [ ] Add detailed change description
- [ ] Update relevant sections
- [ ] Update layout diagram if layout changed
- [ ] Update screenshots (if any)
- [ ] Commit with clear message

---

## 📸 Visual Examples

### Block Producer Node
- Shows all metrics including forging stats
- KES remaining prominently displayed (critical for operational continuity)
- Block adoption rate shows productivity

### Relay Node
- Forging stats hidden (not applicable)
- Focus on connectivity and block propagation
- Peer distribution is key operational metric

### Multi-Node View
- Tab shows all nodes at once
- Quick visual scan of fleet health
- Switch between nodes with Tab or number keys
- Each node maintains own history and alerts

---

## ✅ Validation Checklist

When making TUI changes:
- [ ] All metrics still visible
- [ ] Columns balanced visually
- [ ] Colors remain consistent
- [ ] Keyboard shortcuts still work
- [ ] Layout works at 80x24 minimum
- [ ] Sparklines updated appropriately
- [ ] Help text updated
- [ ] Documentation (this file) updated
- [ ] Code compiles with no warnings
- [ ] Visual regression tested

---

## 🤝 Contributing TUI Changes

1. **Plan the change:** Describe what/why/how
2. **Sketch the layout:** ASCII diagram of new arrangement
3. **Implement:** Make code changes
4. **Test:** Verify at multiple terminal sizes
5. **Update docs:** This file must be updated
6. **Commit:** With clear message referencing this file
7. **Tag release:** Follow semantic versioning

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v0.1.21 | 2026-02-02 | Block sparkline removed, memory gauge added (60/40 with epoch) |
| v0.1.20 | 2026-02-02 | Epoch progress moved to full-width section below header |
| v0.1.19 | 2026-02-02 | Side-by-side sparklines, optimized height, cleaner layout |
| v0.1.18 | 2026-02-02 | Code review fixes, no TUI changes |
| v0.1.17 | 2026-02-02 | Version bump, no TUI changes |
| v0.1.16 | 2026-02-02 | 3-column layout, 50+ metrics, 7 color themes |
| v0.1.15 | Earlier | Previous design (2 columns) |

---

**Document maintained by:** Claw Daddy (@clawd)  
**Last reviewed:** 2026-02-02  
**Status:** 🟢 Current and accurate
