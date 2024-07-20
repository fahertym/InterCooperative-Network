// icn_core/src/config.rs

use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use icn_types::{IcnResult, IcnError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub shard_count: u64,
    pub consensus_threshold: f64,
    pub consensus_quorum: f64,
    pub network_port: u16,
}

impl Config {
    pub fn load(path: &str) -> IcnResult<Self> {
        let config_str = fs::read_to_string(path).map_err(|e| IcnError::ConfigError(e.to_string()))?;
        let config: Config = serde_json::from_str(&config_str).map_err(|e| IcnError::ConfigError(e.to_string()))?;
        Ok(config)
    }

    pub fn save(&self, path: &str) -> IcnResult<()> {
        let config_str = serde_json::to_string_pretty(self).map_err(|e| IcnError::ConfigError(e.to_string()))?;
        fs::write(path, config_str).map_err(|e| IcnError::ConfigError(e.to_string()))?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            shard_count: 4,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
        }
    }
}
