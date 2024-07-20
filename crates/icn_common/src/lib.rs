use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Blockchain error: {0}")]
    BlockchainError(String),
    #[error("Consensus error: {0}")]
    ConsensusError(String),
    #[error("Currency error: {0}")]
    CurrencyError(String),
    #[error("Governance error: {0}")]
    GovernanceError(String),
    #[error("Identity error: {0}")]
    IdentityError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Node management error: {0}")]
    NodeManagementError(String),
    #[error("Sharding error: {0}")]
    ShardingError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("VM error: {0}")]
    VMError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

pub type CommonResult<T> = std::result::Result<T, CommonError>;
