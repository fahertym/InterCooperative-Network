// File: crates/icn_common/src/lib.rs

pub mod error;
pub mod bit_utils;

pub use crate::error::{IcnError, IcnResult};

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier}; // Importing the necessary traits and types
use rand_chacha::ChaCha20Rng;
use rand::RngCore; // Importing necessary traits for random number generation
use rand::SeedableRng;
use std::collections::HashMap;

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
        let message = format!("{}{}{}{}", self.from, self.to, self.amount, self.timestamp);
        let signature = keypair.sign(message.as_bytes()).to_bytes().to_vec();
        self.signature = Some(signature);
        Ok(())
    }

    pub fn verify(&self) -> IcnResult<bool> {
        if let Some(signature) = &self.signature {
            let message = format!("{}{}{}{}", self.from, self.to, self.amount, self.timestamp);
            let public_key = PublicKey::from_bytes(&self.from.as_bytes())
                .map_err(|e| IcnError::Identity(format!("PublicKey conversion failed: {}", e)))?;
            let signature = Signature::from_bytes(signature)
                .map_err(|e| IcnError::Identity(format!("Signature conversion failed: {}", e)))?;
            public_key
                .verify(message.as_bytes(), &signature)
                .map_err(|e| IcnError::Identity(format!("Signature verification failed: {}", e)))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_fee(&self) -> f64 {
        // Simplified fee calculation; in a real implementation, fees would be more complex
        0.01
    }
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
    use ed25519_dalek::Signer;
    use rand_chacha::ChaCha20Rng;
    use rand::SeedableRng;

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

    #[test]
    fn test_transaction_signing_and_verification() {
        let mut rng = ChaCha20Rng::seed_from_u64(0); // Use a deterministic seed for testing
        let keypair: Keypair = Keypair::generate(&mut rng);

        let mut tx = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        tx.sign(&keypair).expect("Signing failed");
        assert!(tx.signature.is_some());

        let verified = tx.verify().expect("Verification failed");
        assert!(verified);
    }
}
