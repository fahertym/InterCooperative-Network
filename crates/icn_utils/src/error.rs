// icn_utils/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IcnError {
    #[error("Invalid transaction")]
    InvalidTransaction,
    // Add other error types as needed
}

pub type Result<T> = std::result::Result<T, IcnError>;
