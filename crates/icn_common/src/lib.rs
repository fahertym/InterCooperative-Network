// File: icn_common/src/lib.rs

pub mod error;

pub use crate::error::{IcnError, IcnResult};

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub shard_count: u64,
    pub consensus_threshold: f64,
    pub consensus_quorum: f64,
    pub network_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: i64,
    pub signature: Option<Vec<u8>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
    pub timestamp: i64,
    pub zkp: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalCategory {
    Economic,
    Technical,
    Social,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CurrencyType {
    BasicNeeds,
    Education,
    Environmental,
    Community,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStats {
    pub node_count: usize,
    pub total_transactions: usize,
    pub active_proposals: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_equality() {
        let tx1 = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        let tx2 = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        assert_eq!(tx1, tx2);
    }

    #[test]
    fn test_currency_type_equality() {
        assert_eq!(CurrencyType::BasicNeeds, CurrencyType::BasicNeeds);
        assert_ne!(CurrencyType::BasicNeeds, CurrencyType::Education);
    }

    #[test]
    fn test_proposal_status() {
        let status1 = ProposalStatus::Active;
        let status2 = ProposalStatus::Passed;
        assert_ne!(status1, status2);
    }

    #[test]
    fn test_network_stats() {
        let stats = NetworkStats {
            node_count: 5,
            total_transactions: 100,
            active_proposals: 3,
        };
        assert_eq!(stats.node_count, 5);
        assert_eq!(stats.total_transactions, 100);
        assert_eq!(stats.active_proposals, 3);
    }
}