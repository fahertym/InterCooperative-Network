use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod error;
pub use error::{IcnError, IcnResult};

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

    pub fn sign(&mut self, keypair: &Keypair) -> IcnResult<()> {
        let message = self.hash().as_bytes().to_vec();
        let signature = keypair.sign(&message);
        self.signature = Some(signature.to_bytes().to_vec());
        Ok(())
    }

    pub fn verify(&self, public_key: &PublicKey) -> IcnResult<bool> {
        let message = self.hash().as_bytes().to_vec();
        if let Some(signature) = &self.signature {
            let signature = Signature::from_bytes(signature)
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
    use rand::rngs::OsRng;

    #[test]
    fn test_transaction_sign_and_verify() {
        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );

        tx.sign(&keypair).unwrap();
        assert!(tx.verify(&keypair.public).unwrap());

        // Test with incorrect public key
        let wrong_keypair: Keypair = Keypair::generate(&mut csprng);
        assert!(!tx.verify(&wrong_keypair.public).unwrap());
    }

    #[test]
    fn test_block_creation_and_hashing() {
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );
        let block = Block::new(1, vec![tx], "previous_hash".to_string());
        
        assert_eq!(block.index, 1);
        assert!(!block.hash.is_empty());
        assert_eq!(block.previous_hash, "previous_hash");
        assert_eq!(block.transactions.len(), 1);

        // Ensure hash changes when block content changes
        let mut modified_block = block.clone();
        modified_block.timestamp += 1;
        assert_ne!(block.hash, modified_block.hash());
    }

    #[test]
    fn test_proposal_and_vote() {
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
}