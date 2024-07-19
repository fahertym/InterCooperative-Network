use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Blockchain error: {0}")]
    BlockchainError(String),
    
    #[error("Consensus error: {0}")]
    ConsensusError(String),
    
    #[error("Sharding error: {0}")]
    ShardingError(String),
    
    #[error("VM error: {0}")]
    VmError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Smart contract error: {0}")]
    SmartContractError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("Communication error: {0}")]
    CommunicationError(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Currency error: {0}")]
    CurrencyError(String),

    #[error("Identity error: {0}")]
    IdentityError(String),

    #[error("Governance error: {0}")]
    GovernanceError(String),
}

pub type Result<T> = std::result::Result<T, Error>;