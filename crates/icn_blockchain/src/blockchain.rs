use icn_common::{Block, Transaction, IcnResult, IcnError, Hashable};
use std::collections::HashMap;
use chrono::Utc;
use log::{info, warn, error};
use std::sync::{Arc, RwLock};

/// Represents a blockchain, maintaining a list of blocks and pending transactions.
pub struct Blockchain {
    chain: Arc<RwLock<Vec<Block>>>,
    pending_transactions: Arc<RwLock<Vec<Transaction>>>,
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
            chain: Arc::new(RwLock::new(vec![Block::genesis()])),
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
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
    pub async fn add_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        if self.transaction_validator.validate(&transaction, self) {
            self.pending_transactions.write().await.push(transaction);
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
    pub async fn create_block(&self) -> IcnResult<Block> {
        let chain = self.chain.read().await;
        let previous_block = chain.last()
            .ok_or_else(|| IcnError::Blockchain("No previous block found".to_string()))?;

        let new_block = Block {
            index: chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: self.pending_transactions.read().await.clone(),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(), // Will be set in calculate_hash
        };

        let new_block = self.calculate_hash(new_block)?;
        self.chain.write().await.push(new_block.clone());
        self.pending_transactions.write().await.clear();

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
    pub async fn validate_chain(&self) -> bool {
        let chain = self.chain.read().await;
        for i in 1..chain.len() {
            let current_block = &chain[i];
            let previous_block = &chain[i - 1];

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
    pub async fn get_latest_block(&self) -> Option<Block> {
        self.chain.read().await.last().cloned()
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
    pub async fn get_block_by_index(&self, index: u64) -> Option<Block> {
        self.chain.read().await.get(index as usize).cloned()
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
    pub async fn get_block_by_hash(&self, hash: &str) -> Option<Block> {
        self.chain.read().await.iter().find(|block| block.hash == hash).cloned()
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

    #[tokio::test]
    async fn test_blockchain_creation() {
        let blockchain = Blockchain::new(Arc::new(MockTransactionValidator));
        assert_eq!(blockchain.chain.read().await.len(), 1);
        assert_eq!(blockchain.chain.read().await[0].index, 0);
    }

    #[tokio::test]
    async fn test_add_block() {
        let blockchain = Blockchain::new(Arc::new(MockTransactionValidator));
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            Utc::now().timestamp(),
        );
        blockchain.add_transaction(transaction).await.unwrap();
        assert!(blockchain.create_block().await.is_ok());
        assert_eq!(blockchain.chain.read().await.len(), 2);
    }

    #[tokio::test]
    async fn test_blockchain_validity() {
        let blockchain = Blockchain::new(Arc::new(MockTransactionValidator));
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            Utc::now().timestamp(),
        );
        blockchain.add_transaction(transaction).await.unwrap();
        blockchain.create_block().await.unwrap();
        assert!(blockchain.validate_chain().await);
    }
}
