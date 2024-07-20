// Declare the modules
mod block;
mod transaction;
mod asset_tokenization;

// Now use the items from these modules
use block::Block;
use transaction::Transaction;
use asset_tokenization::{AssetToken, AssetRegistry};

use std::collections::HashMap;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub asset_registry: AssetRegistry,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: Vec::new(),
            asset_registry: AssetRegistry::new(),
        }
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), String> {
        if self.is_valid_block(&block) {
            self.chain.push(block);
            Ok(())
        } else {
            Err("Invalid block".to_string())
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        // Add validation logic here if needed
        self.pending_transactions.push(transaction);
        Ok(())
    }

    pub fn create_block(&mut self, author: String) -> Result<(), String> {
        let previous_block = self.chain.last().ok_or("No previous block found")?;
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

    fn is_valid_block(&self, block: &Block) -> bool {
        // Implement block validation logic
        true
    }

    pub fn create_asset_token(&mut self, name: String, description: String, owner: String) -> Result<String, String> {
        self.asset_registry.create_token(name, description, owner, serde_json::json!({}))
    }

    pub fn transfer_asset_token(&mut self, token_id: &str, new_owner: &str) -> Result<(), String> {
        self.asset_registry.transfer_token(token_id, new_owner.to_string())
    }
}

// Re-export the types that should be publicly accessible
pub use self::block::Block;
pub use self::transaction::Transaction;
pub use self::asset_tokenization::{AssetToken, AssetRegistry};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);
    }

    // Add more tests here
}