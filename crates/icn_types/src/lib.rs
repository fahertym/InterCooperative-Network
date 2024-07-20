use serde::{Serialize, Deserialize};
use thiserror::Error;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Error, Debug, Serialize, Deserialize)]
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
    VM(String),
    #[error("General error: {0}")]
    General(String),
}

pub type IcnResult<T> = std::result::Result<T, IcnError>;

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
    AssetToken(String),
    Bond(String),
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
    pub proposal_type: ProposalType,
    pub category: ProposalCategory,
    pub required_quorum: f64,
    pub execution_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Implemented,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalCategory {
    Constitutional,
    Economic,
    Technical,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    PersonalDevice,
    CooperativeServer,
    GovernmentServer,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetToken {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub last_transferred: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bond {
    pub id: String,
    pub name: String,
    pub description: String,
    pub issuer: String,
    pub face_value: f64,
    pub maturity_date: DateTime<Utc>,
    pub interest_rate: f64,
    pub owner: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Currency {
    pub currency_type: CurrencyType,
    pub total_supply: f64,
    pub creation_date: DateTime<Utc>,
    pub last_issuance: DateTime<Utc>,
    pub issuance_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub address: String,
    pub balances: HashMap<CurrencyType, f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Address(String),
    List(Vec<Value>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Opcode {
    Push(Value),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
    And,
    Or,
    Not,
    Store(String),
    Load(String),
    Call(String),
    Return,
    JumpIf(usize),
    Jump(usize),
    Vote(String),
    AllocateResource(String),
    UpdateReputation(String),
    CreateProposal,
    GetProposalStatus,
    Emit(String),
}