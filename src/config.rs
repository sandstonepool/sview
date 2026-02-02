//! Configuration handling for sview
//!
//! Configuration is done via CLI arguments and/or environment variables.
//! CLI arguments take precedence over environment variables.

use clap::Parser;
use std::time::Duration;

/// A TUI for monitoring Cardano nodes
#[derive(Parser, Debug, Clone)]
#[command(name = "sview")]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Display name for the node
    #[arg(short, long, env = "NODE_NAME", default_value = "Cardano Node")]
    pub node_name: String,

    /// Cardano network (mainnet, preprod, preview, etc.)
    #[arg(long, env = "CARDANO_NETWORK", default_value = "mainnet")]
    pub network: String,

    /// Prometheus metrics host
    #[arg(long, env = "PROM_HOST", default_value = "127.0.0.1")]
    pub prom_host: String,

    /// Prometheus metrics port
    #[arg(short, long, env = "PROM_PORT", default_value_t = 12798)]
    pub prom_port: u16,

    /// Request timeout in seconds
    #[arg(long, env = "PROM_TIMEOUT", default_value_t = 3)]
    pub prom_timeout_secs: u64,

    /// Refresh interval in seconds
    #[arg(short, long, env = "REFRESH_INTERVAL", default_value_t = 2)]
    pub refresh_interval_secs: u64,

    /// History length for sparklines (number of data points to keep)
    #[arg(long, env = "HISTORY_LENGTH", default_value_t = 60)]
    pub history_length: usize,
}

impl Config {
    /// Load configuration from CLI arguments and environment variables
    pub fn load() -> Self {
        Self::parse()
    }

    /// Get the request timeout as Duration
    pub fn prom_timeout(&self) -> Duration {
        Duration::from_secs(self.prom_timeout_secs)
    }

    /// Get the refresh interval as Duration
    pub fn refresh_interval(&self) -> Duration {
        Duration::from_secs(self.refresh_interval_secs)
    }

    /// Get the full Prometheus metrics URL
    pub fn metrics_url(&self) -> String {
        format!("http://{}:{}/metrics", self.prom_host, self.prom_port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node_name: "Cardano Node".to_string(),
            network: "mainnet".to_string(),
            prom_host: "127.0.0.1".to_string(),
            prom_port: 12798,
            prom_timeout_secs: 3,
            refresh_interval_secs: 2,
            history_length: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.prom_port, 12798);
        assert_eq!(config.prom_host, "127.0.0.1");
    }

    #[test]
    fn test_metrics_url() {
        let config = Config::default();
        assert!(config.metrics_url().contains("12798"));
        assert!(config.metrics_url().contains("/metrics"));
    }
}
