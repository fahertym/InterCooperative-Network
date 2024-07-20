// File: crates/icn_common/src/lib.rs

use thiserror::Error;

#[derive(Error, Debug)]
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
    Serialization(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
}

pub type CommonResult<T> = Result<T, CommonError>;

pub mod logging {
    use log::{info, warn, error, debug};

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

    pub fn to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
        serde_json::to_string(value)
    }

    pub fn from_json<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, serde_json::Error> {
        serde_json::from_str(json)
    }
}