use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use thiserror::Error;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    PersonalDevice,
    CooperativeServer,
    GovernmentServer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub address: String,
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
    #[error("API error: {0}")]
    Api(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type IcnResult<T> = std::result::Result<T, IcnError>;

pub trait Validator<T> {
    fn validate(&self, item: &T) -> bool;
}

pub trait Hashable {
    fn hash(&self) -> String;
}

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

impl Transaction {
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType, timestamp: i64) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            timestamp,
            signature: None,
        }
    }

    pub fn sign(&mut self, private_key: &[u8]) -> IcnResult<()> {
        use ed25519_dalek::{Keypair, Signer};
        let keypair = Keypair::from_bytes(private_key)
            .map_err(|e| IcnError::Identity(format!("Invalid private key: {}", e)))?;
        let message = self.hash().as_bytes().to_vec();
        let signature = keypair.sign(&message);
        self.signature = Some(signature.to_bytes().to_vec());
        Ok(())
    }

    pub fn verify(&self, public_key: &[u8]) -> IcnResult<bool> {
        use ed25519_dalek::{PublicKey, Verifier};
        let public_key = PublicKey::from_bytes(public_key)
            .map_err(|e| IcnError::Identity(format!("Invalid public key: {}", e)))?;
        let message = self.hash().as_bytes().to_vec();
        if let Some(signature) = &self.signature {
            let signature = ed25519_dalek::Signature::from_bytes(signature)
                .map_err(|e| IcnError::Identity(format!("Invalid signature: {}", e)))?;
            Ok(public_key.verify(&message, &signature).is_ok())
        } else {
            Ok(false)
        }
    }
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
        };
        block.hash = block.hash();
        block
    }

    pub fn genesis() -> Self {
        Block::new(0, Vec::new(), "0".repeat(64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );
        let hash = tx.hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_block_hash() {
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );
        let block = Block::new(1, vec![tx], "previous_hash".to_string());
        let hash = block.hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
        assert_eq!(block.hash, hash);
    }

    #[test]
    fn test_transaction_sign_and_verify() {
        use ed25519_dalek::{Keypair, Signer};
        use rand::rngs::OsRng;

        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );

        tx.sign(keypair.to_bytes().as_ref()).unwrap();
        assert!(tx.verify(keypair.public.as_bytes()).unwrap());

        // Test with wrong public key
        let wrong_keypair: Keypair = Keypair::generate(&mut csprng);
        assert!(!tx.verify(wrong_keypair.public.as_bytes()).unwrap());
    }

    #[test]
    fn test_proposal() {
        let proposal = Proposal {
            id: "prop1".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };

        assert_eq!(proposal.status, ProposalStatus::Active);
        assert_eq!(proposal.proposal_type, ProposalType::Constitutional);
        assert_eq!(proposal.category, ProposalCategory::Economic);
    }

    #[test]
    fn test_vote() {
        let vote = Vote {
            voter: "Bob".to_string(),
            proposal_id: "prop1".to_string(),
            in_favor: true,
            weight: 1.0,
            timestamp: Utc::now(),
        };

        assert_eq!(vote.voter, "Bob");
        assert_eq!(vote.proposal_id, "prop1");
        assert!(vote.in_favor);
        assert_eq!(vote.weight, 1.0);
    }

    #[test]
    fn test_node() {
        let node = Node {
            id: "node1".to_string(),
            node_type: NodeType::PersonalDevice,
            address: "127.0.0.1:8000".to_string(),
        };

        assert_eq!(node.id, "node1");
        assert!(matches!(node.node_type, NodeType::PersonalDevice));
        assert_eq!(node.address, "127.0.0.1:8000");
    }

    #[test]
    fn test_icn_error() {
        let error = IcnError::Blockchain("Test error".to_string());
        assert_eq!(error.to_string(), "Blockchain error: Test error");

        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let icn_error: IcnError = io_error.into();
        assert!(matches!(icn_error, IcnError::Io(_)));
    }

    #[test]
    fn test_validator_trait() {
        struct AlwaysValidValidator;
        impl Validator<Transaction> for AlwaysValidValidator {
            fn validate(&self, _item: &Transaction) -> bool {
                true
            }
        }

        let validator = AlwaysValidValidator;
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );

        assert!(validator.validate(&tx));
    }

    #[test]
    fn test_hashable_trait() {
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );
        let tx_hash = tx.hash();
        assert!(!tx_hash.is_empty());
        assert_eq!(tx_hash.len(), 64);

        let block = Block::new(1, vec![tx], "previous_hash".to_string());
        let block_hash = block.hash();
        assert!(!block_hash.is_empty());
        assert_eq!(block_hash.len(), 64);
    }
}