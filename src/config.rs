//! Configuration handling for sview
//!
//! Configuration can be done via:
//! 1. Config file (~/.config/sview/config.toml) - supports multiple nodes
//! 2. CLI arguments - single node mode, overrides config file
//! 3. Environment variables - fallback for CLI defaults
//!
//! When a config file exists and no CLI host/port is specified, multi-node mode is used.

use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// A TUI for monitoring Cardano nodes
#[derive(Parser, Debug, Clone)]
#[command(name = "sview")]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Display name for the node
    #[arg(short, long, env = "NODE_NAME")]
    pub node_name: Option<String>,

    /// Cardano network (mainnet, preprod, preview, etc.)
    #[arg(long, env = "CARDANO_NETWORK")]
    pub network: Option<String>,

    /// Prometheus metrics host
    #[arg(long, env = "PROM_HOST")]
    pub prom_host: Option<String>,

    /// Prometheus metrics port
    #[arg(short, long, env = "PROM_PORT")]
    pub prom_port: Option<u16>,

    /// Request timeout in seconds
    #[arg(long, env = "PROM_TIMEOUT", default_value_t = 3)]
    pub prom_timeout_secs: u64,

    /// Refresh interval in seconds
    #[arg(short, long, env = "REFRESH_INTERVAL", default_value_t = 2)]
    pub refresh_interval_secs: u64,

    /// History length for sparklines (number of data points to keep)
    #[arg(long, env = "HISTORY_LENGTH", default_value_t = 60)]
    pub history_length: usize,

    /// Epoch length in slots (432000 for mainnet, 86400 for testnets)
    #[arg(long, env = "EPOCH_LENGTH", default_value_t = 432000)]
    pub epoch_length: u64,

    /// Path to config file (default: ~/.config/sview/config.toml)
    #[arg(short, long, env = "SVIEW_CONFIG")]
    pub config: Option<PathBuf>,

    /// Export collected metrics to CSV file and exit
    #[arg(long, value_name = "FILE")]
    pub export: Option<PathBuf>,
}

/// Configuration file structure (TOML)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileConfig {
    /// Global settings
    #[serde(default)]
    pub global: GlobalConfig,

    /// Node definitions (array of tables: [[nodes]] in TOML)
    #[serde(default)]
    pub nodes: Vec<NodeConfig>,
}

/// Global settings in config file
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct GlobalConfig {
    /// Default network for all nodes
    #[serde(default = "default_network")]
    pub network: String,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Refresh interval in seconds
    #[serde(default = "default_refresh")]
    pub refresh_interval_secs: u64,

    /// History length for sparklines
    #[serde(default = "default_history")]
    pub history_length: usize,

    /// Epoch length in slots
    #[serde(default = "default_epoch_length")]
    pub epoch_length: u64,

    /// Color theme for TUI
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            network: default_network(),
            timeout_secs: default_timeout(),
            refresh_interval_secs: default_refresh(),
            history_length: default_history(),
            epoch_length: default_epoch_length(),
            theme: default_theme(),
        }
    }
}

/// Individual node configuration
#[derive(Debug, Clone, Deserialize)]
pub struct NodeConfig {
    /// Display name for this node
    pub name: String,

    /// Prometheus metrics host
    #[serde(default = "default_host")]
    pub host: String,

    /// Prometheus metrics port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Node role (relay, bp/block-producer)
    #[serde(default)]
    pub role: NodeRole,

    /// Network override for this node
    pub network: Option<String>,

    /// Node software version (e.g., "10.1.4")
    /// If not specified, sview will try to auto-detect from metrics
    pub version: Option<String>,
}

/// Node role for display/behavior hints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeRole {
    #[default]
    Relay,
    #[serde(alias = "block-producer")]
    Bp,
}

impl std::fmt::Display for NodeRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeRole::Relay => write!(f, "Relay"),
            NodeRole::Bp => write!(f, "BP"),
        }
    }
}

// Default value functions for serde
fn default_network() -> String {
    "mainnet".to_string()
}
fn default_timeout() -> u64 {
    3
}
fn default_refresh() -> u64 {
    2
}
fn default_history() -> usize {
    60
}
fn default_epoch_length() -> u64 {
    432000
}
fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    12798
}
fn default_theme() -> String {
    "dark-default".to_string()
}

/// Runtime configuration for a single node
#[derive(Debug, Clone)]
pub struct NodeRuntimeConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub role: NodeRole,
    pub network: String,
    /// Optional node version from config
    pub version: Option<String>,
}

impl NodeRuntimeConfig {
    /// Get the full Prometheus metrics URL
    #[allow(dead_code)]
    pub fn metrics_url(&self) -> String {
        format!("http://{}:{}/metrics", self.host, self.port)
    }
}

/// Resolved application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// All configured nodes
    pub nodes: Vec<NodeRuntimeConfig>,

    /// Global settings
    pub timeout: Duration,
    pub refresh_interval: Duration,
    pub history_length: usize,
    pub epoch_length: u64,

    /// Export path (if --export was specified)
    pub export_path: Option<PathBuf>,
}

impl AppConfig {
    /// Load configuration from CLI, environment, and config file
    pub fn load() -> Self {
        let args = CliArgs::parse();

        // Determine config file path
        let config_path = args.config.clone().or_else(default_config_path);

        // Try to load config file
        let file_config = config_path
            .and_then(|p| fs::read_to_string(&p).ok())
            .and_then(|s| toml::from_str::<FileConfig>(&s).ok())
            .unwrap_or_default();

        // Check if we should use CLI single-node mode or config file multi-node mode
        let cli_node_specified = args.prom_host.is_some() || args.prom_port.is_some();

        let nodes = if cli_node_specified || file_config.nodes.is_empty() {
            // Single-node mode from CLI
            vec![NodeRuntimeConfig {
                name: args.node_name.unwrap_or_else(|| "Cardano Node".to_string()),
                host: args.prom_host.unwrap_or_else(|| "127.0.0.1".to_string()),
                port: args.prom_port.unwrap_or(12798),
                role: NodeRole::Relay,
                network: args
                    .network
                    .unwrap_or_else(|| file_config.global.network.clone()),
                version: None, // CLI mode doesn't support version specification
            }]
        } else {
            // Multi-node mode from config file
            let configured_nodes: Vec<NodeRuntimeConfig> = file_config
                .nodes
                .iter()
                .map(|n| NodeRuntimeConfig {
                    name: n.name.clone(),
                    host: n.host.clone(),
                    port: n.port,
                    role: n.role,
                    network: n
                        .network
                        .clone()
                        .unwrap_or_else(|| file_config.global.network.clone()),
                    version: n.version.clone(),
                })
                .collect();

            if configured_nodes.is_empty() {
                eprintln!("Error: Configuration has no nodes defined");
                eprintln!("Please add at least one [[nodes]] section to your config file");
                std::process::exit(1);
            }

            configured_nodes
        };

        // Use CLI args for global settings, with file config as fallback
        let timeout_secs = args.prom_timeout_secs;
        let refresh_secs = args.refresh_interval_secs;
        let history_length = args.history_length;
        let epoch_length = args.epoch_length;

        Self {
            nodes,
            timeout: Duration::from_secs(timeout_secs),
            refresh_interval: Duration::from_secs(refresh_secs),
            history_length,
            epoch_length,
            export_path: args.export,
        }
    }

    /// Check if running in multi-node mode
    #[allow(dead_code)]
    pub fn is_multi_node(&self) -> bool {
        self.nodes.len() > 1
    }
}

/// Get the default config file path
fn default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("sview").join("config.toml"))
}

/// Legacy Config struct for backward compatibility with App
#[derive(Debug, Clone)]
pub struct Config {
    pub node_name: String,
    pub network: String,
    pub prom_host: String,
    pub prom_port: u16,
    pub prom_timeout_secs: u64,
    pub refresh_interval_secs: u64,
    pub history_length: usize,
    pub epoch_length: u64,
    /// Optional node version from config
    pub version: Option<String>,
}

impl Config {
    /// Create from NodeRuntimeConfig and AppConfig
    pub fn from_node(node: &NodeRuntimeConfig, app_config: &AppConfig) -> Self {
        Self {
            node_name: node.name.clone(),
            network: node.network.clone(),
            prom_host: node.host.clone(),
            prom_port: node.port,
            prom_timeout_secs: app_config.timeout.as_secs(),
            refresh_interval_secs: app_config.refresh_interval.as_secs(),
            history_length: app_config.history_length,
            epoch_length: app_config.epoch_length,
            version: node.version.clone(),
        }
    }

    /// Get the request timeout as Duration
    pub fn prom_timeout(&self) -> Duration {
        Duration::from_secs(self.prom_timeout_secs)
    }

    /// Get the refresh interval as Duration
    #[allow(dead_code)]
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
            epoch_length: 432000,
            version: None,
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

    #[test]
    fn test_parse_config_file() {
        let toml = r#"
[global]
network = "preprod"
refresh_interval_secs = 5

[[nodes]]
name = "Relay 1"
host = "10.0.0.1"
port = 12798
role = "relay"

[[nodes]]
name = "Block Producer"
host = "10.0.0.2"
port = 12798
role = "bp"
"#;
        let config: FileConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.global.network, "preprod");
        assert_eq!(config.global.refresh_interval_secs, 5);
        assert_eq!(config.nodes.len(), 2);
        assert_eq!(config.nodes[0].name, "Relay 1");
        assert_eq!(config.nodes[1].role, NodeRole::Bp);
    }

    #[test]
    fn test_node_role_aliases() {
        let toml = r#"
[[nodes]]
name = "BP"
role = "block-producer"
"#;
        let config: FileConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.nodes[0].role, NodeRole::Bp);
    }
}
