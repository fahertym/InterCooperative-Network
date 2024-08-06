// File: icn_blockchain/src/blockchain.rs

use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::sync::{Arc, RwLock};
use crate::transaction_validator::TransactionValidator;
use crate::consensus::ConsensusAlgorithm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
}

impl Block {
    pub fn genesis() -> Self {
        Block {
            index: 0,
            timestamp: Utc::now().timestamp(),
            transactions: vec![],
            previous_hash: String::from("0"),
            hash: String::from("genesis_hash"),
        }
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(self.timestamp.to_string());
        hasher.update(serde_json::to_string(&self.transactions).unwrap());
        hasher.update(&self.previous_hash);
        format!("{:x}", hasher.finalize())
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.timestamp == other.timestamp
            && self.transactions == other.transactions
            && self.previous_hash == other.previous_hash
            && self.hash == other.hash
    }
}

/// Represents a blockchain, maintaining a list of blocks and pending transactions.
pub struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    transaction_validator: Arc<TransactionValidator>,
    consensus: Arc<RwLock<dyn ConsensusAlgorithm>>,
}

impl Blockchain {
    /// Creates a new blockchain with a genesis block.
    ///
    /// # Arguments
    ///
    /// * `transaction_validator` - A validator for verifying transactions.
    /// * `consensus` - The consensus algorithm.
    pub fn new(transaction_validator: Arc<TransactionValidator>, consensus: Arc<RwLock<dyn ConsensusAlgorithm>>) -> Self {
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

        new_block.hash = new_block.calculate_hash();
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

            if current_block.hash != current_block.calculate_hash() {
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

    /// Returns the balance of an account for a specific currency type.
    ///
    /// # Arguments
    ///
    /// * `address` - The account address.
    /// * `currency_type` - The type of currency.
    ///
    /// # Returns
    ///
    /// `IcnResult<f64>` representing the balance or an error.
    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        // Simplified example; this should be handled through a currency system in a real implementation
        let balance = self.chain.iter()
            .flat_map(|block| block.transactions.iter())
            .filter(|tx| tx.from == address || tx.to == address)
            .fold(0.0, |acc, tx| {
                if tx.from == address {
                    acc - tx.amount
                } else if tx.to == address {
                    acc + tx.amount
                } else {
                    acc
                }
            });

        Ok(balance)
    }
}
