// File: crates/icn_blockchain/src/lib.rs

use icn_common::{Block, Transaction, IcnResult, IcnError, Hashable};
use std::collections::HashMap;
use chrono::Utc;
use log::{info, warn, error};
use std::sync::{Arc, RwLock};

pub struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    transaction_validator: Arc<dyn TransactionValidator>,
    total_transactions: u64,
}

pub trait TransactionValidator: Send + Sync {
    fn validate(&self, transaction: &Transaction, blockchain: &Blockchain) -> IcnResult<()>;
}

impl Blockchain {
    pub fn new(transaction_validator: Arc<dyn TransactionValidator>) -> IcnResult<Self> {
        Ok(Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: Vec::new(),
            transaction_validator,
            total_transactions: 0,
        })
    }

    pub fn start(&self) -> IcnResult<()> {
        info!("Blockchain started");
        Ok(())
    }

    pub fn stop(&self) -> IcnResult<()> {
        info!("Blockchain stopped");
        Ok(())
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
        self.transaction_validator.validate(&transaction, self)?;
        self.pending_transactions.push(transaction);
        self.total_transactions += 1;
        Ok(())
    }

    pub fn create_block(&mut self) -> IcnResult<Block> {
        let previous_block = self.chain.last()
            .ok_or_else(|| IcnError::Blockchain("No previous block found".to_string()))?;

        let mut new_block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: self.pending_transactions.clone(),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(),
        };

        new_block.hash = new_block.calculate_hash();

        self.chain.push(new_block.clone());
        self.pending_transactions.clear();

        Ok(new_block)
    }

    pub fn get_total_transactions(&self) -> u64 {
        self.total_transactions
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn get_block_by_index(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        self.chain.iter().find(|block| block.hash == hash)
    }

    pub fn validate_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != current_block.calculate_hash() {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }
}

impl Block {
    pub fn genesis() -> Self {
        let mut genesis_block = Block {
            index: 0,
            timestamp: Utc::now().timestamp(),
            transactions: Vec::new(),
            previous_hash: String::from("0"),
            hash: String::new(),
        };
        genesis_block.hash = genesis_block.calculate_hash();
        genesis_block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.index.to_string().as_bytes());
        hasher.update(self.timestamp.to_string().as_bytes());
        for transaction in &self.transactions {
            hasher.update(transaction.hash().as_bytes());
        }
        hasher.update(self.previous_hash.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct MockTransactionValidator;

    impl TransactionValidator for MockTransactionValidator {
        fn validate(&self, _transaction: &Transaction, _blockchain: &Blockchain) -> IcnResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_blockchain_creation() {
        let validator = Arc::new(MockTransactionValidator);
        let blockchain = Blockchain::new(validator).unwrap();
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.chain[0].index, 0);
    }

    #[test]
    fn test_add_block() {
        let validator = Arc::new(MockTransactionValidator);
        let mut blockchain = Blockchain::new(validator).unwrap();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        blockchain.add_transaction(transaction).unwrap();
        let new_block = blockchain.create_block().unwrap();
        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(new_block.index, 1);
        assert_eq!(new_block.transactions.len(), 1);
    }

    #[test]
    fn test_blockchain_validity() {
        let validator = Arc::new(MockTransactionValidator);
        let mut blockchain = Blockchain::new(validator).unwrap();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        blockchain.add_transaction(transaction).unwrap();
        blockchain.create_block().unwrap();
        assert!(blockchain.validate_chain());

        // Tamper with a block
        blockchain.chain[1].transactions[0].amount = 200.0;
        assert!(!blockchain.validate_chain());
    }
}