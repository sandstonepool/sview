use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileConfig {
    #[serde(default)]
    pub global: GlobalConfig,

    #[serde(default, rename = "node")]
    pub nodes: Vec<NodeConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GlobalConfig {
    pub network: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
}

fn main() {
    let toml_str = fs::read_to_string("test_config.toml").unwrap();
    println!("TOML content:\n{}\n", toml_str);
    
    match toml::from_str::<FileConfig>(&toml_str) {
        Ok(config) => {
            println!("Parsed config:");
            println!("  Global network: {}", config.global.network);
            println!("  Nodes count: {}", config.nodes.len());
            for node in &config.nodes {
                println!("    - {}: {}:{}", node.name, node.host, node.port);
            }
        }
        Err(e) => println!("Parse error: {}", e),
    }
}
