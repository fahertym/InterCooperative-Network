use thiserror::Error;
use serde::{Serialize, Deserialize};
use log::{info, warn, error, debug};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum CommonError {
    #[error("Blockchain error: {0}")]
    Blockchain(String),
    #[error("Consensus error: {0}")]
    Consensus(String),
    #[error("Currency error: {0}")]
    Currency(String),
    #[error("Governance error: {0}")]
    Governance(String),
    #[error("Identity error: {0}")]
    Identity(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Sharding error: {0}")]
    Sharding(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("VM error: {0}")]
    VM(String),
    #[error("API error: {0}")]
    API(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("I/O error: {0}")]
    IO(String),
}

pub type CommonResult<T> = Result<T, CommonError>;

pub mod logging {
    use super::*;

    pub fn log_info(message: &str) {
        info!("{}", message);
    }

    pub fn log_warn(message: &str) {
        warn!("{}", message);
    }

    pub fn log_error(message: &str) {
        error!("{}", message);
    }

    pub fn log_debug(message: &str) {
        debug!("{}", message);
    }
}

pub mod serialization {
    use serde::{Serialize, Deserialize};
    use super::CommonResult;

    pub fn to_json<T: Serialize>(value: &T) -> CommonResult<String> {
        serde_json::to_string(value).map_err(|e| CommonError::Serialization(e.to_string()))
    }

    pub fn from_json<T: for<'de> Deserialize<'de>>(json: &str) -> CommonResult<T> {
        serde_json::from_str(json).map_err(|e| CommonError::Serialization(e.to_string()))
    }
}

pub mod config {
    use serde::{Serialize, Deserialize};
    use std::fs;
    use super::{CommonResult, CommonError};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Config {
        pub shard_count: u64,
        pub consensus_threshold: f64,
        pub consensus_quorum: f64,
        pub network_port: u16,
    }

    impl Config {
        pub fn load(path: &str) -> CommonResult<Self> {
            let config_str = fs::read_to_string(path)
                .map_err(|e| CommonError::IO(e.to_string()))?;
            let config: Config = serde_json::from_str(&config_str)
                .map_err(|e| CommonError::Serialization(e.to_string()))?;
            Ok(config)
        }

        pub fn save(&self, path: &str) -> CommonResult<()> {
            let config_str = serde_json::to_string_pretty(self)
                .map_err(|e| CommonError::Serialization(e.to_string()))?;
            fs::write(path, config_str)
                .map_err(|e| CommonError::IO(e.to_string()))?;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_error() {
        let error = CommonError::Blockchain("Test error".to_string());
        assert_eq!(error.to_string(), "Blockchain error: Test error");
    }

    #[test]
    fn test_serialization() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestStruct {
            field: String,
        }

        let test_struct = TestStruct {
            field: "test".to_string(),
        };

        let json = serialization::to_json(&test_struct).unwrap();
        let deserialized: TestStruct = serialization::from_json(&json).unwrap();

        assert_eq!(test_struct, deserialized);
    }

    #[test]
    fn test_config() {
        let config = config::Config::default();
        assert_eq!(config.shard_count, 4);
        assert_eq!(config.consensus_threshold, 0.66);
        assert_eq!(config.consensus_quorum, 0.51);
        assert_eq!(config.network_port, 8080);
    }
}