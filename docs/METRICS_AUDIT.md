# Metrics Audit: cardano-node Source vs sview

This document compares the actual Prometheus metrics exposed by cardano-node (from source code analysis) against sview's metrics.rs parsing implementation.

## Metric Naming Convention

From `trace-dispatcher/src/Cardano/Logging/Tracer/EKG.hs`:
- **Prefix**: `tcMetricsPrefix` (default: `cardano_node_metrics_`)
- **Suffix by type**:
  - `IntM name value` → `{prefix}{name}_int`
  - `DoubleM name value` → `{prefix}{name}_real`
  - `CounterM name value` → `{prefix}{name}_counter`
  - `PrometheusM name labels` → `{prefix}{name}` (with labels)

## Complete Metric Reference (from cardano-node source)

### Chain/Block Metrics (ChainDB.hs)
Emitted on `AddedToCurrentChain` and `SwitchedToAFork` events:

| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `density` | `cardano_node_metrics_density_real` | DoubleM | `density` | ✅ Matches |
| `slotNum` | `cardano_node_metrics_slotNum_int` | IntM | `slot_num` | ✅ Matches |
| `blockNum` | `cardano_node_metrics_blockNum_int` | IntM | `block_height` | ✅ Matches |
| `slotInEpoch` | `cardano_node_metrics_slotInEpoch_int` | IntM | `slot_in_epoch` | ✅ Matches |
| `epoch` | `cardano_node_metrics_epoch_int` | IntM | `epoch` | ✅ Matches |
| `forks` | `cardano_node_metrics_forks_counter` | CounterM | `forks` | ✅ Matches |

### Mempool Metrics (Consensus.hs)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `txsInMempool` | `cardano_node_metrics_txsInMempool_int` | IntM | `mempool_txs` | ✅ Matches |
| `mempoolBytes` | `cardano_node_metrics_mempoolBytes_int` | IntM | `mempool_bytes` | ✅ Matches |
| `txsProcessedNum` | `cardano_node_metrics_txsProcessedNum_counter` | CounterM | `tx_processed` | ⚠️ Type mismatch - sview expects `_int` |
| `txsSyncDuration` | `cardano_node_metrics_txsSyncDuration_int` | IntM | - | ❌ Not parsed |

### Block Fetch Client Metrics (Consensus.hs)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `blockfetchclient.blockdelay` | `cardano_node_metrics_blockfetchclient_blockdelay_s` | DoubleM | `block_delay_s` | ⚠️ Different suffix (source has `_real`, sview expects `_s`) |
| `blockfetchclient.blocksize` | `cardano_node_metrics_blockfetchclient_blocksize_int` | IntM | - | ❌ Not parsed |
| `blockfetchclient.lateblocks` | `cardano_node_metrics_blockfetchclient_lateblocks_counter` | CounterM | `blocks_late` | ⚠️ sview expects no suffix |
| `blockfetchclient.blockdelay.cdfOne` | `cardano_node_metrics_blockfetchclient_blockdelay_cdfOne_real` | DoubleM | `block_delay_cdf_1s` | ⚠️ Different suffix |
| `blockfetchclient.blockdelay.cdfThree` | `cardano_node_metrics_blockfetchclient_blockdelay_cdfThree_real` | DoubleM | `block_delay_cdf_3s` | ⚠️ Different suffix |
| `blockfetchclient.blockdelay.cdfFive` | `cardano_node_metrics_blockfetchclient_blockdelay_cdfFive_real` | DoubleM | `block_delay_cdf_5s` | ⚠️ Different suffix |

### Server Metrics (Consensus.hs)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `served.block` | `cardano_node_metrics_served_block_count_counter` | CounterM | `blocks_served` | ⚠️ sview expects `_int` suffix |
| `served.header` | `cardano_node_metrics_served_header_counter` | CounterM | - | ❌ Not parsed |

### Peer/Connection Metrics (P2P.hs, Consensus.hs)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `connectedPeers` | `cardano_node_metrics_connectedPeers_int` | IntM | `peers_connected` | ✅ Matches |
| `peerSelection.Cold` | `cardano_node_metrics_peerSelection_Cold_int` | IntM | `p2p.cold_peers` | ⚠️ Different naming (sview: `peerSelection_cold`) |
| `peerSelection.Warm` | `cardano_node_metrics_peerSelection_Warm_int` | IntM | `p2p.warm_peers` | ⚠️ Different naming |
| `peerSelection.Hot` | `cardano_node_metrics_peerSelection_Hot_int` | IntM | `p2p.hot_peers` | ⚠️ Different naming |

### Connection Manager Metrics (P2P.hs)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| (from ConnectionManagerCounters) | `cardano_node_metrics_connectionManager_incomingConns` | IntM | `incoming_connections` | ✅ Matches |
| | `cardano_node_metrics_connectionManager_outgoingConns` | IntM | `outgoing_connections` | ✅ Matches |
| | `cardano_node_metrics_connectionManager_duplexConns` | IntM | `full_duplex_connections` | ✅ Matches |
| | `cardano_node_metrics_connectionManager_unidirectionalConns` | IntM | `unidirectional_connections` | ✅ Matches |

### Forging Metrics (ForgingStats.hs, Startup.hs)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `nodeCannotForge` | `cardano_node_metrics_nodeCannotForge_int` | IntM | - | ❌ Not parsed |
| `nodeIsLeader` | `cardano_node_metrics_nodeIsLeader_int` | IntM | `is_leader` | ⚠️ Different field (sview expects Forge_node_is_leader) |
| `blocksForged` | `cardano_node_metrics_blocksForged_int` | IntM | `blocks_adopted` | ⚠️ Different naming |
| `slotsMissed` | `cardano_node_metrics_slotsMissed_int` | IntM | `missed_slots` | ⚠️ Different naming (sview: slotsMissedNum) |
| `forging_enabled` | `cardano_node_metrics_forging_enabled_int` | IntM | - | ❌ Not parsed (could derive node type) |

### KES Metrics (KESInfo.hs - needs verification)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `currentKESPeriod` | `cardano_node_metrics_currentKESPeriod_int` | IntM | `kes_period` | ✅ Matches |
| `remainingKESPeriods` | `cardano_node_metrics_remainingKESPeriods_int` | IntM | `kes_remaining` | ✅ Matches |
| `operationalCertificateExpiryKESPeriod` | `cardano_node_metrics_operationalCertificateExpiryKESPeriod_int` | IntM | `kes_periods_per_cert` | ✅ Matches |

### Ledger Metrics (LedgerMetrics.hs)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `utxoSize` | `cardano_node_metrics_utxoSize_int` | IntM | - | ❌ Not parsed |
| `delegMapSize` | `cardano_node_metrics_delegMapSize_int` | IntM | - | ❌ Not parsed |

### Startup Metrics (Startup.hs)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `node.start.time` | `cardano_node_metrics_node_start_time_int` | IntM | `node_start_time` | ⚠️ Different naming (sview: nodeStartTime) |

### GC/RTS Metrics (Resources - from GHC RTS)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `RTS.gcLiveBytes` | `cardano_node_metrics_RTS_gcLiveBytes_int` | IntM | `memory_used` | ✅ Matches |
| `RTS.gcHeapBytes` | `cardano_node_metrics_RTS_gcHeapBytes_int` | IntM | `memory_heap` | ✅ Matches |
| `RTS.gcMinorNum` | `cardano_node_metrics_RTS_gcMinorNum_int` | IntM | `gc_minor` | ✅ Matches |
| `RTS.gcMajorNum` | `cardano_node_metrics_RTS_gcMajorNum_int` | IntM | `gc_major` | ✅ Matches |
| `Mem.resident` | `cardano_node_metrics_Mem_resident_int` | IntM | (fallback) | ✅ Matches |

### CPU Metrics
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `rts_gc_cpu_ms` | `rts_gc_cpu_ms` | - | `cpu_ms` | ⚠️ No prefix in sview |
| `RTS.cpuNs` | `cardano_node_metrics_RTS_cpuNs_int` | IntM | `cpu_ms` (converted) | ✅ Matches |

### Submission Metrics (Consensus.hs)
| Source Name | Prometheus Name | Type | sview Field | Status |
|-------------|-----------------|------|-------------|--------|
| `submissions.submitted` | `cardano_node_metrics_submissions_submitted_counter` | CounterM | - | ❌ Not parsed |
| `submissions.accepted` | `cardano_node_metrics_submissions_accepted_counter` | CounterM | - | ❌ Not parsed |
| `submissions.rejected` | `cardano_node_metrics_submissions_rejected_counter` | CounterM | - | ❌ Not parsed |

## Issues Found

### Critical Issues
1. **txsProcessedNum**: sview expects `_int` suffix but source emits `_counter`
2. **served.block**: sview expects `served_block_count_int` but source emits `served_block_counter` or similar

### Naming Mismatches
1. **nodeStartTime**: sview expects `nodeStartTime_int`, source emits `node_start_time_int`
2. **peerSelection**: sview uses lowercase (`peerSelection_cold`), source uses CamelCase (`peerSelection.Cold`)
3. **slotsMissedNum**: sview expects `slotsMissedNum_int`, source emits `slotsMissed_int`

### Missing in sview
1. `utxoSize` - useful for monitoring
2. `delegMapSize` - useful for monitoring
3. `forging_enabled` - could help identify block producers
4. `txsSyncDuration` - mempool sync timing
5. `submissions.*` - transaction submission stats

### Legacy/Fallback Handling
sview correctly handles:
- Multiple naming variants for uptime (`nodeStartTime_int`, `upTime_ns`)
- Multiple naming variants for CPU (`rts_gc_cpu_ms`, `RTS_cpuNs_int`)
- Fallback from `RTS_gcLiveBytes_int` to `Mem_resident_int`

## Recommendations

1. **Update metric name matching** to use the exact names from cardano-node source
2. **Add missing metrics** that are useful for monitoring (utxoSize, forging_enabled)
3. **Handle Counter vs Int** - some metrics changed from Int to Counter
4. **Add test coverage** for actual Prometheus output from real nodes
5. **Document metric sources** - link to cardano-node source files

## Reference Files in cardano-node

- `trace-dispatcher/src/Cardano/Logging/Tracer/EKG.hs` - metric formatting
- `cardano-node/src/Cardano/Node/Tracing/Tracers/Consensus.hs` - mempool, blockfetch metrics
- `cardano-node/src/Cardano/Node/Tracing/Tracers/ChainDB.hs` - chain metrics (blockNum, slotNum, epoch)
- `cardano-node/src/Cardano/Node/Tracing/Tracers/ForgingStats.hs` - forging metrics
- `cardano-node/src/Cardano/Node/Tracing/Tracers/P2P.hs` - peer selection metrics
- `cardano-node/src/Cardano/Node/Tracing/Tracers/LedgerMetrics.hs` - ledger metrics
- `cardano-node/src/Cardano/Node/Tracing/Tracers/Startup.hs` - startup metrics

---
Generated: 2026-02-02
Source: cardano-node master branch analysis
