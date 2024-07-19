// crates/icn_blockchain/src/lib.rs

mod block;
mod transaction;
mod blockchain;
mod asset_tokenization;

pub use block::Block;
pub use transaction::Transaction;
pub use blockchain::Blockchain;
pub use asset_tokenization::{AssetToken, AssetRegistry};

use icn_core::error::{Error, Result};
use icn_consensus::PoCConsensus;
use icn_currency::CurrencyType;

pub struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    asset_registry: AssetRegistry,
    consensus: PoCConsensus,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: Vec::new(),
            asset_registry: AssetRegistry::new(),
            consensus: PoCConsensus::new(),
        }
    }

    pub fn add_block(&mut self, block: Block, votes: &[(&str, bool)]) -> Result<()> {
        if self.consensus.validate_block(&block.hash, votes)? {
            if self.is_valid_block(&block) {
                self.chain.push(block);
                self.pending_transactions.clear();
                Ok(())
            } else {
                Err(Error::BlockchainError("Invalid block".to_string()))
            }
        } else {
            Err(Error::BlockchainError("Block not approved by consensus".to_string()))
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        if self.is_valid_transaction(&transaction) {
            self.pending_transactions.push(transaction);
            Ok(())
        } else {
            Err(Error::BlockchainError("Invalid transaction".to_string()))
        }
    }

    fn is_valid_block(&self, block: &Block) -> bool {
        // Implement block validation logic
        block.previous_hash == self.chain.last().unwrap().hash
            && block.hash == block.calculate_hash()
            && block.timestamp > self.chain.last().unwrap().timestamp
    }

    fn is_valid_transaction(&self, transaction: &Transaction) -> bool {
        // Implement transaction validation logic
        let sender_balance = self.get_balance(&transaction.from).unwrap_or(0.0);
        sender_balance >= transaction.amount
    }

    pub fn get_balance(&self, address: &str) -> Result<f64> {
        let balance = self.chain.iter()
            .flat_map(|block| &block.transactions)
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

    pub fn create_asset_token(&mut self, name: String, description: String, owner: String) -> Result<AssetToken> {
        self.asset_registry.create_token(name, description, owner)
    }

    pub fn transfer_asset_token(&mut self, token_id: &str, new_owner: &str) -> Result<()> {
        self.asset_registry.transfer_token(token_id, new_owner.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);
    }

    #[test]
    fn test_add_transaction() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction::new("Alice".to_string(), "Bob".to_string(), 100.0, CurrencyType::BasicNeeds);
        assert!(blockchain.add_transaction(transaction).is_ok());
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction::new("Alice".to_string(), "Bob".to_string(), 100.0, CurrencyType::BasicNeeds);
        blockchain.add_transaction(transaction).unwrap();
        
        let new_block = Block::new(
            blockchain.chain.len() as u64,
            blockchain.pending_transactions.clone(),
            blockchain.chain.last().unwrap().hash.clone(),
        );

        // Assume all members vote in favor for this test
        let votes = vec![("Alice", true), ("Bob", true), ("Charlie", true)];
        assert!(blockchain.add_block(new_block, &votes).is_ok());
        assert_eq!(blockchain.chain.len(), 2);
        assert!(blockchain.pending_transactions.is_empty());
    }
}