// File: icn_utils/src/lib.rs

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
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

pub type IcnResult<T> = Result<T, IcnError>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum CurrencyType {
    BasicNeeds,
    Education,
    Environmental,
    Community,
    Volunteer,
    Storage,
    Processing,
    Energy,
    Luxury,
    Service,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: i64,
    pub signature: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub status: ProposalStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Implemented,
}

pub mod crypto {
    use sha2::{Sha256, Digest};

    pub fn hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    pub fn hex_encode(data: &[u8]) -> String {
        hex::encode(data)
    }

    pub fn hex_decode(s: &str) -> IcnResult<Vec<u8>> {
        hex::decode(s).map_err(|e| IcnError::Blockchain(format!("Hex decode error: {}", e)))
    }
}

pub mod validation {
    use super::*;

    pub fn is_valid_address(address: &str) -> bool {
        // Implement address validation logic
        address.len() == 64 && address.chars().all(|c| c.is_ascii_hexdigit())
    }

    pub fn is_valid_transaction(transaction: &Transaction) -> bool {
        // Implement transaction validation logic
        transaction.amount > 0.0 && is_valid_address(&transaction.from) && is_valid_address(&transaction.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_types() {
        assert_ne!(CurrencyType::BasicNeeds, CurrencyType::Education);
        assert_eq!(CurrencyType::Custom("Test".to_string()), CurrencyType::Custom("Test".to_string()));
    }

    #[test]
    fn test_crypto_functions() {
        let data = b"Hello, ICN!";
        let hash = crypto::hash(data);
        assert_eq!(hash.len(), 32);

        let hex = crypto::hex_encode(&hash);
        assert_eq!(hex.len(), 64);

        let decoded = crypto::hex_decode(&hex).unwrap();
        assert_eq!(decoded, hash);
    }

    #[test]
    fn test_validation_functions() {
        assert!(validation::is_valid_address(&"a".repeat(64)));
        assert!(!validation::is_valid_address(&"a".repeat(63)));

        let valid_transaction = Transaction {
            from: "a".repeat(64),
            to: "b".repeat(64),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        assert!(validation::is_valid_transaction(&valid_transaction));

        let invalid_transaction = Transaction {
            from: "invalid",
            to: "b".repeat(64),
            amount: 0.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        assert!(!validation::is_valid_transaction(&invalid_transaction));
    }
}