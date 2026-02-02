//! Critical alerts system for monitoring node health
//!
//! Detects problematic state transitions and alerts operators to issues.

use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tracing::debug;

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARN"),
            AlertSeverity::Critical => write!(f, "CRIT"),
        }
    }
}

/// A single alert event
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Alert {
    pub timestamp: u64,
    pub node_name: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
}

#[allow(dead_code)]
impl Alert {
    /// Format alert for display
    pub fn display(&self) -> String {
        format!("[{}] {} - {}", self.severity, self.title, self.message)
    }

    /// Format alert for file logging
    pub fn log_format(&self) -> String {
        let datetime = timestamp_to_iso8601(self.timestamp);
        format!(
            "{} | {} | {} | {} | {}",
            datetime, self.node_name, self.severity, self.title, self.message
        )
    }
}

/// Alert manager for a single node
#[allow(dead_code)]
pub struct AlertManager {
    node_name: String,
    log_file: Option<PathBuf>,
    recent_alerts: VecDeque<Alert>,
    max_recent: usize,

    // State tracking for deduplication
    last_kes_warning: Option<u64>,
    last_peer_warning: Option<u64>,
    last_sync_warning: Option<u64>,
    last_height_stall_warning: Option<u64>,
}

#[allow(dead_code)]
impl AlertManager {
    /// Create a new alert manager for a node
    pub fn new(node_name: &str) -> Self {
        let log_file = get_alerts_log_path(node_name);

        Self {
            node_name: node_name.to_string(),
            log_file,
            recent_alerts: VecDeque::new(),
            max_recent: 50, // Keep last 50 alerts in memory

            last_kes_warning: None,
            last_peer_warning: None,
            last_sync_warning: None,
            last_height_stall_warning: None,
        }
    }

    /// Check KES periods and alert if critical
    pub fn check_kes_expiry(&mut self, kes_remaining: Option<u64>) {
        if let Some(remaining) = kes_remaining {
            if remaining < 5 {
                // Only warn once per hour
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if let Some(last_warn) = self.last_kes_warning {
                    if now - last_warn < 3600 {
                        return;
                    }
                }

                let alert = Alert {
                    timestamp: now,
                    node_name: self.node_name.clone(),
                    severity: AlertSeverity::Critical,
                    title: "KES Expiry Critical".to_string(),
                    message: format!(
                        "KES periods remaining: {} (renew certificate immediately)",
                        remaining
                    ),
                };

                self.add_alert(alert);
                self.last_kes_warning = Some(now);
            }
        }
    }

    /// Check peer count and alert if low
    pub fn check_peer_count(&mut self, peers: Option<u64>) {
        if let Some(count) = peers {
            if count < 2 {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if let Some(last_warn) = self.last_peer_warning {
                    if now - last_warn < 300 {
                        // 5 min cooldown
                        return;
                    }
                }

                let alert = Alert {
                    timestamp: now,
                    node_name: self.node_name.clone(),
                    severity: if count == 0 {
                        AlertSeverity::Critical
                    } else {
                        AlertSeverity::Warning
                    },
                    title: "Low Peer Count".to_string(),
                    message: format!("Only {} peer(s) connected", count),
                };

                self.add_alert(alert);
                self.last_peer_warning = Some(now);
            }
        }
    }

    /// Check sync progress and alert if degraded
    pub fn check_sync_progress(&mut self, sync_progress: Option<f64>) {
        if let Some(progress) = sync_progress {
            if progress < 95.0 {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if let Some(last_warn) = self.last_sync_warning {
                    if now - last_warn < 600 {
                        // 10 min cooldown
                        return;
                    }
                }

                let alert = Alert {
                    timestamp: now,
                    node_name: self.node_name.clone(),
                    severity: if progress < 90.0 {
                        AlertSeverity::Critical
                    } else {
                        AlertSeverity::Warning
                    },
                    title: "Sync Progress Degraded".to_string(),
                    message: format!("Node is {:.2}% synced", progress),
                };

                self.add_alert(alert);
                self.last_sync_warning = Some(now);
            }
        }
    }

    /// Check for block height stalls
    pub fn check_block_stall(
        &mut self,
        current_height: Option<u64>,
        #[allow(unused_variables)] previous_height: Option<u64>,
        time_since_last_block: Option<u64>,
    ) {
        // Alert if no new blocks in 5+ minutes
        if let Some(age) = time_since_last_block {
            if age > 300 {
                // 5 minutes
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if let Some(last_warn) = self.last_height_stall_warning {
                    if now - last_warn < 600 {
                        // 10 min cooldown
                        return;
                    }
                }

                let alert = Alert {
                    timestamp: now,
                    node_name: self.node_name.clone(),
                    severity: AlertSeverity::Warning,
                    title: "Block Height Stalled".to_string(),
                    message: format!(
                        "No new blocks for {} seconds (height: {})",
                        age,
                        current_height.unwrap_or(0)
                    ),
                };

                self.add_alert(alert);
                self.last_height_stall_warning = Some(now);
            }
        }
    }

    /// Get the most recent critical alert (if any)
    pub fn latest_critical(&self) -> Option<&Alert> {
        self.recent_alerts
            .iter()
            .rev()
            .find(|a| a.severity == AlertSeverity::Critical)
    }

    /// Get all alerts since timestamp
    pub fn alerts_since(&self, timestamp: u64) -> Vec<&Alert> {
        self.recent_alerts
            .iter()
            .filter(|a| a.timestamp >= timestamp)
            .collect()
    }

    /// Add an alert and log it
    fn add_alert(&mut self, alert: Alert) {
        debug!("Alert: {}", alert.display());

        // Log to file
        if let Some(ref log_path) = self.log_file {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                let _ = writeln!(file, "{}", alert.log_format());
            }
        }

        // Keep in memory
        self.recent_alerts.push_back(alert);
        if self.recent_alerts.len() > self.max_recent {
            self.recent_alerts.pop_front();
        }
    }

    /// Clear all alerts (for testing)
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.recent_alerts.clear();
    }
}

/// Get the alerts log file path for a node
#[allow(dead_code)]
fn get_alerts_log_path(node_name: &str) -> Option<PathBuf> {
    dirs::data_dir().map(|p| {
        p.join("sview")
            .join("alerts")
            .join(format!("{}.log", node_name.replace(" ", "_").to_lowercase()))
    })
}

/// Convert Unix timestamp to ISO8601 datetime string
#[allow(dead_code)]
fn timestamp_to_iso8601(ts: u64) -> String {
    let seconds_in_day = ts % 86400;
    let days_since_epoch = ts / 86400;

    // Simple conversion (approximate for readability)
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

    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days in days_in_months {
        if remaining_days < days as i64 {
            break;
        }
        remaining_days -= days as i64;
        month += 1;
    }

    let day = remaining_days + 1;
    let hour = seconds_in_day / 3600;
    let minute = (seconds_in_day % 3600) / 60;
    let second = seconds_in_day % 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

#[allow(clippy::manual_is_multiple_of)]
fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kes_alert() {
        let mut manager = AlertManager::new("Test BP");
        assert!(manager.latest_critical().is_none());

        manager.check_kes_expiry(Some(3));
        assert!(manager.latest_critical().is_some());
        assert_eq!(manager.latest_critical().unwrap().severity, AlertSeverity::Critical);
    }

    #[test]
    fn test_peer_alert() {
        let mut manager = AlertManager::new("Test Relay");
        manager.check_peer_count(Some(0));
        assert!(manager.latest_critical().is_some());
        assert_eq!(
            manager.latest_critical().unwrap().severity,
            AlertSeverity::Critical
        );
    }

    #[test]
    fn test_sync_alert() {
        let mut manager = AlertManager::new("Test Node");
        manager.check_sync_progress(Some(85.0));
        assert!(manager.latest_critical().is_some());
    }

    #[test]
    fn test_no_alert_threshold() {
        let mut manager = AlertManager::new("Test Node");
        manager.check_kes_expiry(Some(20));
        assert!(manager.latest_critical().is_none());

        manager.check_peer_count(Some(5));
        assert!(manager.latest_critical().is_none());

        manager.check_sync_progress(Some(99.9));
        assert!(manager.latest_critical().is_none());
    }
}
