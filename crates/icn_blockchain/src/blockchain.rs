// File: icn_blockchain/src/blockchain.rs

use icn_common::{Block, Transaction, IcnResult, IcnError, Hashable};
use std::collections::HashMap;
use chrono::Utc;
use log::{info, warn, error};
use std::sync::{Arc, RwLock};
use crate::consensus::ConsensusAlgorithm;
use crate::transaction_validator::TransactionValidator;

/// Represents a blockchain, maintaining a list of blocks and pending transactions.
pub struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    transaction_validator: Arc<dyn TransactionValidator>,
    consensus: Arc<RwLock<dyn ConsensusAlgorithm>>,
}

impl Blockchain {
    /// Creates a new blockchain with a genesis block.
    ///
    /// # Arguments
    ///
    /// * `transaction_validator` - A validator for verifying transactions.
    /// * `consensus` - The consensus algorithm.
    pub fn new(transaction_validator: Arc<dyn TransactionValidator>, consensus: Arc<RwLock<dyn ConsensusAlgorithm>>) -> Self {
        Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: Vec::new(),
            transaction_validator,
            consensus,
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
        self.transaction_validator.validate_transaction(&transaction, self)?;
        self.pending_transactions.push(transaction);
        Ok(())
    }

    /// Creates a new block with all pending transactions.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Blockchain` if no previous block is found or if `calculate_hash` fails.
    pub fn create_block(&mut self) -> IcnResult<Block> {
        let previous_block = self.chain.last()
            .ok_or_else(|| IcnError::Blockchain("No previous block found".to_string()))?;

        let mut new_block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: std::mem::take(&mut self.pending_transactions),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(),
        };

        new_block.hash = new_block.hash();
        self.consensus.read().unwrap().validate_block(&new_block)?;

        self.chain.push(new_block.clone());

        Ok(new_block)
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
}
