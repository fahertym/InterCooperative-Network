use icn_common::{Block, Transaction, IcnResult, IcnError, Hashable};
use std::collections::HashMap;
use chrono::Utc;
use log::{info, warn, error};
use std::sync::{Arc, RwLock};

/// Represents a blockchain, maintaining a list of blocks and pending transactions.
pub struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    transaction_validator: Arc<dyn TransactionValidator>,
}

/// A trait for validating transactions within a blockchain.
pub trait TransactionValidator: Send + Sync {
    /// Validates a transaction within the context of a blockchain.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to validate.
    /// * `blockchain` - The blockchain context.
    ///
    /// # Returns
    ///
    /// `true` if the transaction is valid, `false` otherwise.
    fn validate(&self, transaction: &Transaction, blockchain: &Blockchain) -> bool;
}

impl Blockchain {
    /// Creates a new blockchain with a genesis block.
    ///
    /// # Arguments
    ///
    /// * `transaction_validator` - A validator for verifying transactions.
    pub fn new(transaction_validator: Arc<dyn TransactionValidator>) -> Self {
        Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: Vec::new(),
            transaction_validator,
        }
    }

    /// Adds a new transaction to the list of pending transactions.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to add.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Blockchain` if the transaction is invalid.
    pub fn add_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
        if self.transaction_validator.validate(&transaction, self) {
            self.pending_transactions.push(transaction);
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid transaction".to_string()))
        }
    }

    /// Creates a new block with all pending transactions.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Blockchain` if no previous block is found or if `calculate_hash` fails.
    pub fn create_block(&mut self) -> IcnResult<Block> {
        let previous_block = self.chain.last()
            .ok_or_else(|| IcnError::Blockchain("No previous block found".to_string()))?;

        let new_block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: self.pending_transactions.clone(),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(), // Will be set in calculate_hash
        };

        let new_block = self.calculate_hash(new_block)?;
        self.chain.push(new_block.clone());
        self.pending_transactions.clear();

        Ok(new_block)
    }

    fn calculate_hash(&self, mut block: Block) -> IcnResult<Block> {
        block.hash = block.hash();
        Ok(block)
    }

    /// Validates the integrity of the blockchain.
    ///
    /// # Returns
    ///
    /// `true` if the blockchain is valid, `false` otherwise.
    pub fn validate_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != current_block.hash() {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }

    /// Returns the latest block in the blockchain.
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    /// Returns a block by its index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the block.
    ///
    /// # Returns
    ///
    /// The block if found, `None` otherwise.
    pub fn get_block_by_index(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    /// Returns a block by its hash.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the block.
    ///
    /// # Returns
    ///
    /// The block if found, `None` otherwise.
    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        self.chain.iter().find(|block| block.hash == hash)
    }

    /// Starts the blockchain.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn start(&self) -> IcnResult<()> {
        info!("Blockchain started");
        Ok(())
    }

    /// Stops the blockchain.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn stop(&self) -> IcnResult<()> {
        info!("Blockchain stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::CurrencyType;
    use std::sync::Arc;

    struct MockTransactionValidator;

    impl TransactionValidator for MockTransactionValidator {
        fn validate(&self, _transaction: &Transaction, _blockchain: &Blockchain) -> bool {
            true
        }
    }

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new(Arc::new(MockTransactionValidator));
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.chain[0].index, 0);
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = Blockchain::new(Arc::new(MockTransactionValidator));
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            Utc::now().timestamp(),
        );
        blockchain.add_transaction(transaction).unwrap();
        assert!(blockchain.create_block().is_ok());
        assert_eq!(blockchain.chain.len(), 2);
    }

    #[test]
    fn test_blockchain_validity() {
        let mut blockchain = Blockchain::new(Arc::new(MockTransactionValidator));
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            Utc::now().timestamp(),
        );
        blockchain.add_transaction(transaction).unwrap();
        blockchain.create_block().unwrap();
        assert!(blockchain.validate_chain());
    }
}
