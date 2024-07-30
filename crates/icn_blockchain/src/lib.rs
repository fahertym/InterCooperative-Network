// File: icn_blockchain/src/lib.rs

use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: u32,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String, difficulty: u32) -> Self {
        let mut block = Block {
            index,
            timestamp: Utc::now().timestamp(),
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            difficulty,
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(self.timestamp.to_string());
        hasher.update(serde_json::to_string(&self.transactions).unwrap());
        hasher.update(&self.previous_hash);
        hasher.update(self.nonce.to_string());
        format!("{:x}", hasher.finalize())
    }

    pub fn mine(&mut self) {
        let target = (1 << (256 - self.difficulty)) - 1;
        while self.hash.parse::<u128>().unwrap_or(u128::MAX) > target as u128 {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    difficulty: u32,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block::new(0, Vec::new(), String::from("0"), 4);
        Blockchain {
            chain: vec![genesis_block],
            pending_transactions: Vec::new(),
            difficulty: 4,
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
        // Basic transaction validation
        if transaction.amount <= 0.0 {
            return Err(IcnError::Blockchain("Invalid transaction amount".into()));
        }
        self.pending_transactions.push(transaction);
        Ok(())
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) -> IcnResult<()> {
        let reward_transaction = Transaction {
            from: String::from("Network"),
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
            self.get_latest_block().hash.clone(),
            self.difficulty,
        );
        new_block.mine();

        self.chain.push(new_block);
        self.pending_transactions.clear();
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
        }
        true
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> f64 {
        let mut balance = 0.0;
        for block in &self.chain {
            for transaction in &block.transactions {
                if transaction.currency_type == *currency_type {
                    if transaction.from == address {
                        balance -= transaction.amount;
                    }
                    if transaction.to == address {
                        balance += transaction.amount;
                    }
                }
            }
        }
        balance
    }

    pub fn adjust_difficulty(&mut self) {
        // Adjust difficulty every 10 blocks
        if self.chain.len() % 10 == 0 {
            let last_ten_blocks = &self.chain[self.chain.len() - 10..];
            let average_time = last_ten_blocks.windows(2)
                .map(|w| w[1].timestamp - w[0].timestamp)
                .sum::<i64>() / 9;

            if average_time < 30 {
                self.difficulty += 1;
            } else if average_time > 60 && self.difficulty > 1 {
                self.difficulty -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.chain[0].index, 0);
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        blockchain.add_transaction(transaction).unwrap();
        blockchain.mine_pending_transactions("Miner").unwrap();
        assert_eq!(blockchain.chain.len(), 2);
    }

    #[test]
    fn test_blockchain_validity() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        blockchain.add_transaction(transaction).unwrap();
        blockchain.mine_pending_transactions("Miner").unwrap();
        assert!(blockchain.is_chain_valid());

        // Tamper with a block to test invalid chain
        blockchain.chain[1].transactions[0].amount = 100.0;
        assert!(!blockchain.is_chain_valid());
    }

    #[test]
    fn test_get_balance() {
        let mut blockchain = Blockchain::new();
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
            to: "Alice".to_string(),
            amount: 30.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        blockchain.add_transaction(transaction1).unwrap();
        blockchain.add_transaction(transaction2).unwrap();
        blockchain.mine_pending_transactions("Miner").unwrap();

        assert_eq!(blockchain.get_balance("Alice", &CurrencyType::BasicNeeds), -20.0);
        assert_eq!(blockchain.get_balance("Bob", &CurrencyType::BasicNeeds), 20.0);
        assert_eq!(blockchain.get_balance("Miner", &CurrencyType::BasicNeeds), 1.0);
    }

    #[test]
    fn test_adjust_difficulty() {
        let mut blockchain = Blockchain::new();
        let initial_difficulty = blockchain.difficulty;

        // Mine 10 blocks quickly
        for _ in 0..10 {
            blockchain.add_transaction(Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 1.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: Utc::now().timestamp(),
                signature: None,
            }).unwrap();
            blockchain.mine_pending_transactions("Miner").unwrap();
        }

        blockchain.adjust_difficulty();
        assert!(blockchain.difficulty > initial_difficulty, "Difficulty should increase after mining blocks quickly");

        // Reset difficulty
        blockchain.difficulty = initial_difficulty;

        // Mine 10 blocks slowly
        for i in 0..10 {
            blockchain.add_transaction(Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 1.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: Utc::now().timestamp() + i * 100, // Simulate slower block creation
                signature: None,
            }).unwrap();
            blockchain.mine_pending_transactions("Miner").unwrap();
        }

        blockchain.adjust_difficulty();
        assert!(blockchain.difficulty < initial_difficulty, "Difficulty should decrease after mining blocks slowly");
    }

    #[test]
    fn test_invalid_transaction() {
        let mut blockchain = Blockchain::new();
        let invalid_transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: -50.0, // Invalid negative amount
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        assert!(blockchain.add_transaction(invalid_transaction).is_err(), "Should not allow invalid transactions");
    }
}