// File: crates/icn_blockchain/src/lib.rs

use chrono::{DateTime, Utc};
use icn_common::{IcnResult, IcnError, CurrencyType};
use icn_currency::CurrencySystem;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub merkle_root: String,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: &str) -> Self {
        let mut block = Block {
            index,
            timestamp: Utc::now().timestamp(),
            transactions,
            previous_hash: previous_hash.to_string(),
            hash: String::new(),
            nonce: 0,
            merkle_root: String::new(),
        };
        block.merkle_root = block.calculate_merkle_root();
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(&self.timestamp.to_string());
        hasher.update(&self.merkle_root);
        hasher.update(&self.previous_hash);
        hasher.update(&self.nonce.to_string());
        format!("{:x}", hasher.finalize())
    }

    pub fn calculate_merkle_root(&self) -> String {
        let transaction_hashes: Vec<String> = self.transactions
            .iter()
            .map(|tx| {
                let mut hasher = Sha256::new();
                hasher.update(serde_json::to_string(tx).unwrap().as_bytes());
                format!("{:x}", hasher.finalize())
            })
            .collect();

        if transaction_hashes.is_empty() {
            return String::from("0000000000000000000000000000000000000000000000000000000000000000");
        }

        let mut merkle_tree = transaction_hashes;
        while merkle_tree.len() > 1 {
            let mut new_level = Vec::new();
            for chunk in merkle_tree.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0].as_bytes());
                if chunk.len() > 1 {
                    hasher.update(chunk[1].as_bytes());
                } else {
                    hasher.update(chunk[0].as_bytes());
                }
                new_level.push(format!("{:x}", hasher.finalize()));
            }
            merkle_tree = new_level;
        }

        merkle_tree[0].clone()
    }

    pub fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        while &self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
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

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
    currency_system: CurrencySystem,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            difficulty,
            currency_system: CurrencySystem::new(),
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block::new(0, Vec::new(), "0");
        self.chain.push(genesis_block);
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
        if self.validate_transaction(&transaction)? {
            self.pending_transactions.push(transaction);
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid transaction".into()))
        }
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) -> IcnResult<()> {
        let reward_transaction = Transaction {
            from: "Network".to_string(),
            to: miner_address.to_string(),
            amount: 1.0, // Mining reward
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        self.pending_transactions.push(reward_transaction);

        let mut new_block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            &self.get_latest_block().hash,
        );
        new_block.mine(self.difficulty);
        self.add_block(new_block)?;

        self.pending_transactions.clear();
        Ok(())
    }

    fn add_block(&mut self, block: Block) -> IcnResult<()> {
        self.chain.push(block);
        self.update_balances()?;
        Ok(())
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != current_block.calculate_hash() {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }

            if current_block.merkle_root != current_block.calculate_merkle_root() {
                return false;
            }

            if !self.validate_block_transactions(current_block) {
                return false;
            }
        }
        true
    }

    fn validate_block_transactions(&self, block: &Block) -> bool {
        for transaction in &block.transactions {
            if !self.validate_transaction(transaction).unwrap_or(false) {
                return false;
            }
        }
        true
    }

    fn validate_transaction(&self, transaction: &Transaction) -> IcnResult<bool> {
        if transaction.from == "Network" {
            return Ok(true); // Allow mining rewards
        }

        let sender_balance = self.currency_system.get_balance(&transaction.from, &transaction.currency_type)?;
        if sender_balance < transaction.amount {
            return Ok(false);
        }

        // Additional validation logic can be added here (e.g., signature verification)
        if let Some(signature) = &transaction.signature {
            // Implement signature verification logic here
            // For now, we'll assume all signatures are valid
            // In a real implementation, you would verify the signature against the transaction data
            // using the sender's public key
        }

        Ok(true)
    }

    fn update_balances(&mut self) -> IcnResult<()> {
        for transaction in &self.get_latest_block().transactions {
            self.currency_system.process_transaction(transaction)?;
        }
        Ok(())
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currency_system.get_balance(address, currency_type)
    }

    pub fn get_transactions(&self, address: &str) -> Vec<&Transaction> {
        self.chain
            .iter()
            .flat_map(|block| block.transactions.iter())
            .filter(|tx| tx.from == address || tx.to == address)
            .collect()
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        self.chain.iter().find(|block| block.hash == hash)
    }

    pub fn get_block_by_index(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new(2);
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.chain[0].index, 0);
    }

    #[test]
    fn test_add_transaction_and_mine() {
        let mut blockchain = Blockchain::new(2);
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        // Add initial balance for Alice
        blockchain.currency_system.mint("Alice", &CurrencyType::BasicNeeds, 100.0).unwrap();

        assert!(blockchain.add_transaction(transaction).is_ok());
        assert_eq!(blockchain.pending_transactions.len(), 1);

        assert!(blockchain.mine_pending_transactions("Miner").is_ok());
        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(blockchain.pending_transactions.len(), 0);

        // Check balances after mining
        assert_eq!(blockchain.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(blockchain.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(blockchain.get_balance("Miner", &CurrencyType::BasicNeeds).unwrap(), 1.0);
    }

    #[test]
    fn test_blockchain_validity() {
        let mut blockchain = Blockchain::new(2);
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        blockchain.currency_system.mint("Alice", &CurrencyType::BasicNeeds, 100.0).unwrap();

        assert!(blockchain.add_transaction(transaction).is_ok());
        assert!(blockchain.mine_pending_transactions("Miner").is_ok());

        assert!(blockchain.is_chain_valid());

        // Tamper with a block to test invalid chain
        blockchain.chain[1].transactions[0].amount = 100.0;
        assert!(!blockchain.is_chain_valid());
    }

    #[test]
    fn test_get_transactions() {
        let mut blockchain = Blockchain::new(2);
        let transaction1 = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        let transaction2 = Transaction {
            from: "Bob".to_string(),
            to: "Charlie".to_string(),
            amount: 25.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        blockchain.currency_system.mint("Alice", &CurrencyType::BasicNeeds, 100.0).unwrap();
        blockchain.currency_system.mint("Bob", &CurrencyType::BasicNeeds, 50.0).unwrap();

        assert!(blockchain.add_transaction(transaction1).is_ok());
        assert!(blockchain.add_transaction(transaction2).is_ok());
        assert!(blockchain.mine_pending_transactions("Miner").is_ok());

        let alice_transactions = blockchain.get_transactions("Alice");
        assert_eq!(alice_transactions.len(), 1);
        assert_eq!(alice_transactions[0].from, "Alice");

        let bob_transactions = blockchain.get_transactions("Bob");
        assert_eq!(bob_transactions.len(), 2);
    }
}
