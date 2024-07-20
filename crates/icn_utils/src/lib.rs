// File: icn_utils/src/lib.rs

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use thiserror::Error;
use hex;
use icn_types::{IcnResult, IcnError}; // Ensure to use the correct path to import IcnResult and IcnError

#[derive(Error, Debug)]
pub enum IcnUtilsError {
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

pub type IcnUtilsResult<T> = Result<T, IcnUtilsError>;

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

pub fn hex_encode(data: &[u8]) -> String {
    hex::encode(data)
}

pub fn hex_decode(s: &str) -> IcnResult<Vec<u8>> {
    hex::decode(s).map_err(|e| IcnError::Blockchain(format!("Hex decode error: {}", e)))
}
