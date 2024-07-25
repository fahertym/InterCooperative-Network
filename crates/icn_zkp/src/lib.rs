use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod bit_utils;
pub mod error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    AssetToken(String),
    Bond(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: i64,
    pub signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub status: ProposalStatus,
    pub proposal_type: ProposalType,
    pub category: ProposalCategory,
    pub required_quorum: f64,
    pub execution_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Implemented,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalCategory {
    Constitutional,
    Economic,
    Technical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
}

pub trait Hashable {
    fn hash(&self) -> String;
}

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

    #[error("Sharding error: {0}")]
    Sharding(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("VM error: {0}")]
    Vm(String),

    #[error("ZKP error: {0}")]
    ZKP(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type IcnResult<T> = Result<T, IcnError>;

impl Hashable for Block {
    fn hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(&self.timestamp.to_string());
        for transaction in &self.transactions {
            hasher.update(&transaction.hash());
        }
        hasher.update(&self.previous_hash);
        format!("{:x}", hasher.finalize())
    }
}

impl Hashable for Transaction {
    fn hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.from);
        hasher.update(&self.to);
        hasher.update(self.amount.to_string().as_bytes());
        hasher.update(format!("{:?}", self.currency_type).as_bytes());
        hasher.update(self.timestamp.to_string().as_bytes());
        if let Some(signature) = &self.signature {
            hasher.update(signature);
        }
        format!("{:x}", hasher.finalize())
    }
}

pub mod zkp {
    use super::*;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ZKPProof(pub Vec<u8>);

    pub trait ZKPSystem {
        fn create_proof(&self, transaction: &Transaction) -> IcnResult<ZKPProof>;
        fn verify_proof(&self, proof: &ZKPProof, transaction: &Transaction) -> IcnResult<bool>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 1234567890,
            signature: None,
        };
        let hash = tx.hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_block_hash() {
        let tx = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 1234567890,
            signature: None,
        };
        let block = Block {
            index: 1,
            timestamp: 1234567890,
            transactions: vec![tx],
            previous_hash: "previous_hash".to_string(),
            hash: String::new(),
        };
        let hash = block.hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
    }
}