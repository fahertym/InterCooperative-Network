// File: icn_types/src/lib.rs

// Import necessary modules and crates
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Enum representing different types of currencies in the system.
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

/// Struct representing a transaction in the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: i64,
    pub signature: Option<Vec<u8>>,
}

/// Struct representing a block in the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
}

/// Struct representing a proposal in the governance system.
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

/// Enum representing the status of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Implemented,
}

/// Enum representing different types of proposals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
}

/// Enum representing different categories of proposals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalCategory {
    Constitutional,
    Economic,
    Technical,
}

/// Struct representing a vote in the governance system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
}

/// Enum representing different types of nodes in the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    PersonalDevice,
    CooperativeServer,
    GovernmentServer,
}

/// Struct representing a node in the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub address: String,
}

/// Trait for validating an item.
pub trait Validator<T> {
    fn validate(&self, item: &T) -> bool;
}

/// Trait for computing the hash of an item.
pub trait Hashable {
    fn hash(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_type() {
        let currency = CurrencyType::BasicNeeds;
        assert_eq!(currency, CurrencyType::BasicNeeds);
    }

    #[test]
    fn test_transaction() {
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        assert_eq!(transaction.from, "Alice");
    }

    #[test]
    fn test_block() {
        let block = Block {
            index: 0,
            timestamp: Utc::now().timestamp(),
            transactions: vec![],
            previous_hash: "0".to_string(),
            hash: "hash".to_string(),
        };
        assert_eq!(block.index, 0);
    }

    #[test]
    fn test_proposal() {
        let proposal = Proposal {
            id: "1".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now(),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Constitutional,
            required_quorum: 0.5,
            execution_timestamp: None,
        };
        assert_eq!(proposal.id, "1");
    }

    #[test]
    fn test_vote() {
        let vote = Vote {
            voter: "Alice".to_string(),
            proposal_id: "1".to_string(),
            in_favor: true,
            weight: 1.0,
            timestamp: Utc::now(),
        };
        assert_eq!(vote.voter, "Alice");
    }

    #[test]
    fn test_node() {
        let node = Node {
            id: "node1".to_string(),
            node_type: NodeType::PersonalDevice,
            address: "127.0.0.1".to_string(),
        };
        assert_eq!(node.id, "node1");
    }
}
