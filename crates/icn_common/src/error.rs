use thiserror::Error;

#[derive(Error, Debug)]
pub enum IcnError {
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

    #[error("Node management error: {0}")]
    NodeManagement(String),

    #[error("Sharding error: {0}")]
    Sharding(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("VM error: {0}")]
    Vm(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type IcnResult<T> = std::result::Result<T, IcnError>;
