//! Persistent metric history storage
//!
//! This module handles disk persistence of metric snapshots for long-term
//! trend analysis across sessions. Data is stored as compressed JSON files
//! organized by node and date.
//!
//! Storage location: ~/.local/share/sview/history/{node_name}/YYYY/MM/DD.json.gz

use crate::history::MetricsHistory;
use crate::metrics::NodeMetrics;
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Default retention period in days
const DEFAULT_RETENTION_DAYS: u64 = 30;

/// Minimum interval between saved samples (1 hour in seconds)
const MIN_SAMPLE_INTERVAL_SECS: u64 = 3600;

/// A single metric snapshot for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Unix timestamp in seconds
    pub timestamp: u64,
    /// Block height
    pub block_height: Option<u64>,
    /// Slot number
    pub slot_num: Option<u64>,
    /// Current epoch
    pub epoch: Option<u64>,
    /// Slot in epoch
    pub slot_in_epoch: Option<u64>,
    /// Connected peers count
    pub peers_connected: Option<u64>,
    /// Memory usage in bytes
    pub memory_used: Option<u64>,
    /// Transactions in mempool
    pub mempool_txs: Option<u64>,
    /// Mempool size in bytes
    pub mempool_bytes: Option<u64>,
    /// Sync progress (0-100)
    pub sync_progress: Option<f64>,
    /// KES period
    pub kes_period: Option<u64>,
    /// KES remaining periods
    pub kes_remaining: Option<u64>,
}

impl MetricSnapshot {
    /// Create a snapshot from current metrics
    pub fn from_metrics(metrics: &NodeMetrics) -> Self {
        let timestamp = match SystemTime::now()
            .duration_since(UNIX_EPOCH)
        {
            Ok(dur) => dur.as_secs(),
            Err(_) => {
                warn!("System clock error - using epoch fallback for snapshot");
                0  // Fallback to epoch (will be skipped in cleanup)
            }
        };

        Self {
            timestamp,
            block_height: metrics.block_height,
            slot_num: metrics.slot_num,
            epoch: metrics.epoch,
            slot_in_epoch: metrics.slot_in_epoch,
            peers_connected: metrics.peers_connected,
            memory_used: metrics.memory_used,
            mempool_txs: metrics.mempool_txs,
            mempool_bytes: metrics.mempool_bytes,
            sync_progress: metrics.sync_progress,
            kes_period: metrics.kes_period,
            kes_remaining: metrics.kes_remaining,
        }
    }
}

/// Daily file containing hourly samples
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailySnapshots {
    /// Node name for verification
    pub node_name: String,
    /// Snapshots for this day
    pub snapshots: Vec<MetricSnapshot>,
}

/// Storage manager for persistent metric history
pub struct StorageManager {
    /// Base directory for all storage
    base_dir: PathBuf,
    /// Node name (sanitized for filesystem)
    node_name: String,
    /// Retention period
    retention_days: u64,
    /// Last save timestamp (to enforce hourly sampling)
    last_save_timestamp: Option<u64>,
}

impl StorageManager {
    /// Create a new storage manager for a node
    pub fn new(node_name: &str) -> Self {
        let base_dir = get_data_dir();
        let sanitized_name = sanitize_node_name(node_name);

        debug!(
            "Initializing storage manager for '{}' at {:?}",
            sanitized_name, base_dir
        );

        Self {
            base_dir,
            node_name: sanitized_name,
            retention_days: DEFAULT_RETENTION_DAYS,
            last_save_timestamp: None,
        }
    }

    /// Set custom retention period
    #[allow(dead_code)]
    pub fn with_retention_days(mut self, days: u64) -> Self {
        self.retention_days = days;
        self
    }

    /// Get the directory path for a specific date
    fn date_dir(&self, year: u32, month: u32) -> PathBuf {
        self.base_dir
            .join("history")
            .join(&self.node_name)
            .join(format!("{:04}", year))
            .join(format!("{:02}", month))
    }

    /// Get the file path for a specific date
    fn date_file(&self, year: u32, month: u32, day: u32) -> PathBuf {
        self.date_dir(year, month)
            .join(format!("{:02}.json.gz", day))
    }

    /// Get current date components
    fn current_date() -> (u32, u32, u32) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        timestamp_to_date(now)
    }

    /// Save a metric snapshot to disk
    ///
    /// Only saves if enough time has passed since the last save (hourly sampling)
    pub fn save_snapshot(&mut self, metrics: &NodeMetrics) -> Result<bool> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Check if we should save (hourly sampling)
        if let Some(last) = self.last_save_timestamp {
            if now - last < MIN_SAMPLE_INTERVAL_SECS {
                debug!("Skipping save - not enough time elapsed since last save");
                return Ok(false);
            }
        }

        // Don't save if node is disconnected
        if !metrics.connected {
            debug!("Skipping save - node not connected");
            return Ok(false);
        }

        let snapshot = MetricSnapshot::from_metrics(metrics);
        let (year, month, day) = Self::current_date();

        // Ensure directory exists
        let dir = self.date_dir(year, month);
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create storage directory: {:?}", dir))?;

        // Load existing daily file or create new
        let file_path = self.date_file(year, month, day);
        let mut daily = self.load_daily_file(&file_path).unwrap_or_else(|e| {
            debug!("Creating new daily file (previous load failed: {})", e);
            DailySnapshots {
                node_name: self.node_name.clone(),
                snapshots: Vec::new(),
            }
        });

        // Append new snapshot
        daily.snapshots.push(snapshot);

        // Write back
        self.write_daily_file(&file_path, &daily)?;
        self.last_save_timestamp = Some(now);

        info!(
            "Saved metric snapshot for '{}' ({} total samples today)",
            self.node_name,
            daily.snapshots.len()
        );

        Ok(true)
    }

    /// Load historical data to populate MetricsHistory
    ///
    /// Loads up to `max_samples` most recent samples from the last N days
    pub fn load_history(&self, max_samples: usize) -> Result<Vec<MetricSnapshot>> {
        let mut all_snapshots = Vec::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Iterate over the last retention_days
        for days_ago in 0..self.retention_days {
            let target_ts = now.saturating_sub(days_ago * 86400);
            let (year, month, day) = timestamp_to_date(target_ts);
            let file_path = self.date_file(year, month, day);

            if file_path.exists() {
                match self.load_daily_file(&file_path) {
                    Ok(daily) => {
                        let count = daily.snapshots.len();
                        all_snapshots.extend(daily.snapshots);
                        debug!("Loaded {} snapshots from {:?}", count, file_path);
                    }
                    Err(e) => {
                        warn!("Failed to load {:?}: {}", file_path, e);
                    }
                }
            }
        }

        // Sort by timestamp (oldest first) and limit
        all_snapshots.sort_by_key(|s| s.timestamp);
        if all_snapshots.len() > max_samples {
            let skip_count = all_snapshots.len() - max_samples;
            all_snapshots = all_snapshots.into_iter().skip(skip_count).collect();
        }

        info!(
            "Loaded {} historical samples for '{}'",
            all_snapshots.len(),
            self.node_name
        );

        Ok(all_snapshots)
    }

    /// Populate a MetricsHistory from stored data
    pub fn populate_history(&self, history: &mut MetricsHistory, max_samples: usize) -> Result<()> {
        let snapshots = self.load_history(max_samples)?;

        for snapshot in snapshots {
            if let Some(v) = snapshot.block_height {
                history.block_height.push(v as f64);
            }
            if let Some(v) = snapshot.slot_num {
                history.slot_num.push(v as f64);
            }
            if let Some(v) = snapshot.peers_connected {
                history.peers_connected.push(v as f64);
            }
            if let Some(v) = snapshot.memory_used {
                history.memory_used.push(v as f64);
            }
            if let Some(v) = snapshot.mempool_txs {
                history.mempool_txs.push(v as f64);
            }
            if let Some(v) = snapshot.sync_progress {
                history.sync_progress.push(v);
            }
        }

        Ok(())
    }

    /// Clean up old data beyond retention period
    pub fn cleanup_old_data(&self) -> Result<usize> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(self.retention_days * 86400);

        let history_dir = self.base_dir.join("history").join(&self.node_name);
        if !history_dir.exists() {
            return Ok(0);
        }

        let mut removed_count = 0;

        // Walk year directories
        for year_entry in fs::read_dir(&history_dir)? {
            let year_entry = year_entry?;
            if !year_entry.file_type()?.is_dir() {
                continue;
            }

            let year_path = year_entry.path();

            // Walk month directories
            for month_entry in fs::read_dir(&year_path)? {
                let month_entry = month_entry?;
                if !month_entry.file_type()?.is_dir() {
                    continue;
                }

                let month_path = month_entry.path();

                // Check day files
                for day_entry in fs::read_dir(&month_path)? {
                    let day_entry = day_entry?;
                    let day_path = day_entry.path();

                    // Parse date from path
                    if let Some(file_date) = parse_date_from_path(&day_path) {
                        if file_date < cutoff {
                            fs::remove_file(&day_path)?;
                            removed_count += 1;
                            debug!("Removed old data file: {:?}", day_path);
                        }
                    }
                }

                // Remove empty month directory
                if fs::read_dir(&month_path)?.next().is_none() {
                    fs::remove_dir(&month_path)?;
                }
            }

            // Remove empty year directory
            if fs::read_dir(&year_path)?.next().is_none() {
                fs::remove_dir(&year_path)?;
            }
        }

        if removed_count > 0 {
            info!(
                "Cleaned up {} old data files for '{}'",
                removed_count, self.node_name
            );
        }

        Ok(removed_count)
    }

    /// Load a daily file
    fn load_daily_file(&self, path: &std::path::Path) -> Result<DailySnapshots> {
        let file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
        let reader = BufReader::new(file);
        let mut decoder = GzDecoder::new(reader);
        let mut json_str = String::new();
        decoder
            .read_to_string(&mut json_str)
            .with_context(|| format!("Failed to decompress {:?}", path))?;
        let daily: DailySnapshots = serde_json::from_str(&json_str)
            .with_context(|| format!("Failed to parse {:?}", path))?;
        Ok(daily)
    }

    /// Write a daily file
    fn write_daily_file(&self, path: &std::path::Path, daily: &DailySnapshots) -> Result<()> {
        let file = File::create(path).with_context(|| format!("Failed to create {:?}", path))?;
        let writer = BufWriter::new(file);
        let mut encoder = GzEncoder::new(writer, Compression::default());
        let json_str =
            serde_json::to_string(daily).with_context(|| "Failed to serialize snapshots")?;
        encoder
            .write_all(json_str.as_bytes())
            .with_context(|| format!("Failed to write {:?}", path))?;
        encoder.finish()?;
        Ok(())
    }

    /// Export all historical data to CSV
    pub fn export_to_csv(&self, output_path: &std::path::Path) -> Result<usize> {
        let snapshots = self.load_history(usize::MAX)?;

        let mut writer = BufWriter::new(
            File::create(output_path)
                .with_context(|| format!("Failed to create {:?}", output_path))?,
        );

        // Write header
        writeln!(
            writer,
            "timestamp,datetime,block_height,slot_num,epoch,slot_in_epoch,peers_connected,memory_used_bytes,mempool_txs,mempool_bytes,sync_progress,kes_period,kes_remaining"
        )?;

        // Write data rows
        for snapshot in &snapshots {
            let datetime = timestamp_to_iso8601(snapshot.timestamp);
            writeln!(
                writer,
                "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                snapshot.timestamp,
                datetime,
                opt_to_csv(snapshot.block_height),
                opt_to_csv(snapshot.slot_num),
                opt_to_csv(snapshot.epoch),
                opt_to_csv(snapshot.slot_in_epoch),
                opt_to_csv(snapshot.peers_connected),
                opt_to_csv(snapshot.memory_used),
                opt_to_csv(snapshot.mempool_txs),
                opt_to_csv(snapshot.mempool_bytes),
                opt_f64_to_csv(snapshot.sync_progress),
                opt_to_csv(snapshot.kes_period),
                opt_to_csv(snapshot.kes_remaining),
            )?;
        }

        writer.flush()?;
        info!(
            "Exported {} snapshots to {:?}",
            snapshots.len(),
            output_path
        );

        Ok(snapshots.len())
    }
}

/// Get the data directory for sview
fn get_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sview")
}

/// Sanitize node name for use in filesystem paths
fn sanitize_node_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

/// Convert Unix timestamp to (year, month, day)
fn timestamp_to_date(ts: u64) -> (u32, u32, u32) {
    // Simple implementation - doesn't handle all edge cases but works for reasonable dates
    let days_since_epoch = ts / 86400;
    let mut remaining_days = days_since_epoch as i64;

    let mut year = 1970;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days in days_in_months {
        if remaining_days < days {
            break;
        }
        remaining_days -= days;
        month += 1;
    }

    let day = remaining_days as u32 + 1;
    (year, month, day)
}

#[allow(clippy::manual_is_multiple_of)]
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Convert timestamp to ISO8601 datetime string
fn timestamp_to_iso8601(ts: u64) -> String {
    let (year, month, day) = timestamp_to_date(ts);
    let seconds_in_day = ts % 86400;
    let hour = seconds_in_day / 3600;
    let minute = (seconds_in_day % 3600) / 60;
    let second = seconds_in_day % 60;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

/// Parse date from file path and convert to timestamp
fn parse_date_from_path(path: &std::path::Path) -> Option<u64> {
    let file_name = path.file_stem()?.to_str()?;
    let day: u32 = file_name.parse().ok()?;

    let month_dir = path.parent()?;
    let month: u32 = month_dir.file_name()?.to_str()?.parse().ok()?;

    let year_dir = month_dir.parent()?;
    let year: u32 = year_dir.file_name()?.to_str()?.parse().ok()?;

    Some(date_to_timestamp(year, month, day))
}

/// Convert (year, month, day) to Unix timestamp
fn date_to_timestamp(year: u32, month: u32, day: u32) -> u64 {
    let mut days: u64 = 0;

    // Add days for years since 1970
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Add days for months in current year
    let days_in_months: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    for &days_in_month in days_in_months.iter().take((month - 1) as usize) {
        days += days_in_month;
    }

    // Add days in current month (day - 1 because day 1 = 0 extra days)
    days += (day - 1) as u64;

    days * 86400
}

/// Convert Option<u64> to CSV string
fn opt_to_csv(opt: Option<u64>) -> String {
    opt.map(|v| v.to_string()).unwrap_or_default()
}

/// Convert Option<f64> to CSV string
fn opt_f64_to_csv(opt: Option<f64>) -> String {
    opt.map(|v| format!("{:.2}", v)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_metrics() -> NodeMetrics {
        NodeMetrics {
            connected: true,
            block_height: Some(10500000),
            slot_num: Some(125000000),
            epoch: Some(450),
            slot_in_epoch: Some(50000),
            peers_connected: Some(5),
            memory_used: Some(8_000_000_000),
            mempool_txs: Some(10),
            mempool_bytes: Some(5000),
            sync_progress: Some(100.0),
            kes_period: Some(350),
            kes_remaining: Some(42),
            ..Default::default()
        }
    }

    #[test]
    fn test_sanitize_node_name() {
        assert_eq!(sanitize_node_name("My Node"), "my_node");
        assert_eq!(sanitize_node_name("relay-1"), "relay-1");
        assert_eq!(sanitize_node_name("node/test"), "node_test");
        assert_eq!(sanitize_node_name("Node@123"), "node_123");
    }

    #[test]
    fn test_timestamp_to_date() {
        // 2024-01-15
        let (y, m, d) = timestamp_to_date(1705276800);
        assert_eq!((y, m, d), (2024, 1, 15));

        // 2023-12-31
        let (y, m, d) = timestamp_to_date(1704067199);
        assert_eq!((y, m, d), (2023, 12, 31));
    }

    #[test]
    fn test_date_to_timestamp() {
        let ts = date_to_timestamp(2024, 1, 15);
        let (y, m, d) = timestamp_to_date(ts);
        assert_eq!((y, m, d), (2024, 1, 15));
    }

    #[test]
    fn test_timestamp_to_iso8601() {
        let iso = timestamp_to_iso8601(1705276800);
        assert!(iso.starts_with("2024-01-15T"));
    }

    #[test]
    fn test_metric_snapshot_from_metrics() {
        let metrics = create_test_metrics();
        let snapshot = MetricSnapshot::from_metrics(&metrics);

        assert_eq!(snapshot.block_height, Some(10500000));
        assert_eq!(snapshot.peers_connected, Some(5));
        assert_eq!(snapshot.sync_progress, Some(100.0));
        assert!(snapshot.timestamp > 0);
    }

    #[test]
    fn test_storage_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path()); // dirs crate uses HOME

        let mut manager = StorageManager::new("Test Node");
        manager.base_dir = temp_dir.path().to_path_buf();
        manager.last_save_timestamp = None;

        let metrics = create_test_metrics();

        // Save should succeed
        let saved = manager.save_snapshot(&metrics).unwrap();
        assert!(saved);

        // Immediate second save should skip (hourly limit)
        let saved2 = manager.save_snapshot(&metrics).unwrap();
        assert!(!saved2);

        // Load history
        let history = manager.load_history(100).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].block_height, Some(10500000));
    }

    #[test]
    fn test_populate_history() {
        let temp_dir = TempDir::new().unwrap();

        let mut manager = StorageManager::new("Test Node");
        manager.base_dir = temp_dir.path().to_path_buf();

        let metrics = create_test_metrics();
        manager.save_snapshot(&metrics).unwrap();

        let mut history = MetricsHistory::new(100);
        manager.populate_history(&mut history, 100).unwrap();

        assert_eq!(history.block_height.len(), 1);
        assert_eq!(history.peers_connected.len(), 1);
    }

    #[test]
    fn test_csv_export() {
        let temp_dir = TempDir::new().unwrap();

        let mut manager = StorageManager::new("Test Node");
        manager.base_dir = temp_dir.path().to_path_buf();

        let metrics = create_test_metrics();
        manager.save_snapshot(&metrics).unwrap();

        let csv_path = temp_dir.path().join("export.csv");
        let count = manager.export_to_csv(&csv_path).unwrap();
        assert_eq!(count, 1);

        let csv_content = fs::read_to_string(&csv_path).unwrap();
        assert!(csv_content.contains("timestamp,datetime"));
        assert!(csv_content.contains("10500000"));
    }

    #[test]
    fn test_disconnected_not_saved() {
        let temp_dir = TempDir::new().unwrap();

        let mut manager = StorageManager::new("Test Node");
        manager.base_dir = temp_dir.path().to_path_buf();

        let mut metrics = create_test_metrics();
        metrics.connected = false;

        let saved = manager.save_snapshot(&metrics).unwrap();
        assert!(!saved);
    }
}
