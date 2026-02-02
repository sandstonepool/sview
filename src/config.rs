//! Configuration handling for sview
//!
//! All configuration is done via environment variables following the 12-factor app principles.

use std::env;
use std::time::Duration;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Display name for the node
    pub node_name: String,
    /// Cardano network (mainnet, preprod, preview, etc.)
    pub network: String,
    /// Prometheus metrics host
    pub prom_host: String,
    /// Prometheus metrics port
    pub prom_port: u16,
    /// Request timeout
    pub prom_timeout: Duration,
    /// Refresh interval for metrics
    pub refresh_interval: Duration,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let prom_timeout_secs: u64 = env::var("PROM_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        let refresh_secs: u64 = env::var("REFRESH_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);

        Self {
            node_name: env::var("NODE_NAME").unwrap_or_else(|_| "Cardano Node".to_string()),
            network: env::var("CARDANO_NETWORK")
                .or_else(|_| env::var("NETWORK"))
                .unwrap_or_else(|_| "mainnet".to_string()),
            prom_host: env::var("PROM_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            prom_port: env::var("PROM_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(12798),
            prom_timeout: Duration::from_secs(prom_timeout_secs),
            refresh_interval: Duration::from_secs(refresh_secs),
        }
    }

    /// Get the full Prometheus metrics URL
    pub fn metrics_url(&self) -> String {
        format!("http://{}:{}/metrics", self.prom_host, self.prom_port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::from_env();
        assert_eq!(config.prom_port, 12798);
        assert_eq!(config.prom_host, "127.0.0.1");
    }

    #[test]
    fn test_metrics_url() {
        let config = Config::from_env();
        assert!(config.metrics_url().contains("12798"));
        assert!(config.metrics_url().contains("/metrics"));
    }
}
