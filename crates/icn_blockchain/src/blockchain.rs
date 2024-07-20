// crates/icn_blockchain/src/blockchain.rs
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::currency::CurrencyType;
use crate::consensus::PoCConsensus;
use crate::error::{Error, Result};

pub mod block;
pub mod transaction;

pub use block::Block;
pub use transaction::Transaction;

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub asset_tokens: HashMap<String, CurrencyType>,
    pub bonds: HashMap<String, CurrencyType>,
    pub consensus: PoCConsensus,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            asset_tokens: HashMap::new(),
            bonds: HashMap::new(),
            consensus: PoCConsensus::new(0.5, 0.66),
        };
        
        let genesis_block = Block::genesis();
        blockchain.chain.push(genesis_block);
        
        blockchain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        // Add validation logic here if needed
        self.pending_transactions.push(transaction);
        Ok(())
    }

    pub fn create_block(&mut self, author: String) -> Result<()> {
        let previous_block = self.chain.last().ok_or(Error::BlockchainError("No previous block found".to_string()))?;
        let new_block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            previous_block.hash.clone(),
        );
        
        self.chain.push(new_block);
        self.pending_transactions.clear();
        Ok(())
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;
        for block in &self.chain {
            for transaction in &block.transactions {
                if transaction.from == address {
                    balance -= transaction.amount;
                }
                if transaction.to == address {
                    balance += transaction.amount;
                }
            }
        }
        balance
    }

    pub fn validate_chain(&self) -> Result<()> {
        for i in 1..self.chain.len() {
            let previous_block = &self.chain[i - 1];
            let current_block = &self.chain[i];

            if current_block.previous_hash != previous_block.hash {
                return Err(Error::BlockchainError("Invalid previous hash".to_string()));
            }

            if current_block.hash != current_block.calculate_hash() {
                return Err(Error::BlockchainError("Invalid block hash".to_string()));
            }
        }
        Ok(())
    }

    pub fn add_asset_token(&mut self, token_id: String, token: CurrencyType) {
        self.asset_tokens.insert(token_id, token);
    }

    pub fn get_asset_token(&self, token_id: &str) -> Option<&CurrencyType> {
        self.asset_tokens.get(token_id)
    }
}
