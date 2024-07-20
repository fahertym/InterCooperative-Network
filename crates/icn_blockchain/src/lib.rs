use icn_types::{Block, Transaction, CurrencyType};
use icn_common::{CommonError, CommonResult};
use chrono::Utc;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: u32,
    pub mining_reward: f64,
}

impl Blockchain {
    pub fn new() -> CommonResult<Self> {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            difficulty: 4,
            mining_reward: 100.0,
        };

        blockchain.create_genesis_block()?;

        Ok(blockchain)
    }

    fn create_genesis_block(&mut self) -> CommonResult<()> {
        let genesis_block = Block {
            index: 0,
            timestamp: Utc::now().timestamp(),
            transactions: Vec::new(),
            previous_hash: String::from("0"),
            hash: String::new(),
        };

        let hash = self.calculate_hash(&genesis_block)?;
        let mut genesis_block = genesis_block;
        genesis_block.hash = hash;

        self.chain.push(genesis_block);
        Ok(())
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> CommonResult<()> {
        if !self.validate_transaction(&transaction) {
            return Err(CommonError::BlockchainError("Invalid transaction".into()));
        }

        self.pending_transactions.push(transaction);
        Ok(())
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) -> CommonResult<Block> {
        let mut block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: self.pending_transactions.clone(),
            previous_hash: self.get_latest_block()?.hash.clone(),
            hash: String::new(),
        };

        let mut nonce = 0;
        loop {
            let hash = self.calculate_hash_with_nonce(&block, nonce)?;
            if self.is_hash_valid(&hash) {
                block.hash = hash;
                break;
            }
            nonce += 1;
        }

        self.chain.push(block.clone());

        // Reset pending transactions and reward the miner
        self.pending_transactions = vec![Transaction {
            from: String::from("System"),
            to: miner_address.to_string(),
            amount: self.mining_reward,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        }];

        Ok(block)
    }

    fn calculate_hash(&self, block: &Block) -> CommonResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(block.index.to_string());
        hasher.update(block.timestamp.to_string());
        for transaction in &block.transactions {
            hasher.update(transaction.hash());
        }
        hasher.update(&block.previous_hash);
        
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn calculate_hash_with_nonce(&self, block: &Block, nonce: u64) -> CommonResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(block.index.to_string());
        hasher.update(block.timestamp.to_string());
        for transaction in &block.transactions {
            hasher.update(transaction.hash());
        }
        hasher.update(&block.previous_hash);
        hasher.update(nonce.to_string());
        
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn is_hash_valid(&self, hash: &str) -> bool {
        let prefix = "0".repeat(self.difficulty as usize);
        hash.starts_with(&prefix)
    }

    pub fn get_latest_block(&self) -> CommonResult<&Block> {
        self.chain.last().ok_or_else(|| CommonError::BlockchainError("Blockchain is empty".into()))
    }

    pub fn validate_chain(&self) -> CommonResult<bool> {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != self.calculate_hash(current_block)? {
                return Ok(false);
            }

            if current_block.previous_hash != previous_block.hash {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn validate_transaction(&self, transaction: &Transaction) -> bool {
        // In a real implementation, this would check the transaction signature,
        // ensure the sender has sufficient balance, etc.
        // For now, we'll just do a basic check
        transaction.amount > 0.0
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> CommonResult<f64> {
        let mut balance = 0.0;
        for block in &self.chain {
            for transaction in &block.transactions {
                if transaction.from == address && transaction.currency_type == *currency_type {
                    balance -= transaction.amount;
                }
                if transaction.to == address && transaction.currency_type == *currency_type {
                    balance += transaction.amount;
                }
            }
        }
        Ok(balance)
    }

    pub fn adjust_difficulty(&mut self) {
        // This is a simple difficulty adjustment algorithm.
        // In a real-world scenario, this would be more sophisticated.
        if self.chain.len() % 10 == 0 {
            let last_ten_blocks = &self.chain[self.chain.len() - 10..];
            let average_time = last_ten_blocks.windows(2)
                .map(|w| w[1].timestamp - w[0].timestamp)
                .sum::<i64>() / 9;

            if average_time < 55 { // If blocks are being mined too quickly
                self.difficulty += 1;
            } else if average_time > 65 && self.difficulty > 1 { // If blocks are being mined too slowly
                self.difficulty -= 1;
            }
        }
    }
}

impl Transaction {
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.from);
        hasher.update(&self.to);
        hasher.update(self.amount.to_string());
        hasher.update(format!("{:?}", self.currency_type));
        hasher.update(self.timestamp.to_string());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new().unwrap();
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.chain[0].index, 0);
    }

    #[test]
    fn test_add_transaction() {
        let mut blockchain = Blockchain::new().unwrap();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        assert!(blockchain.add_transaction(transaction).is_ok());
        assert_eq!(blockchain.pending_transactions.len(), 1);
    }

    #[test]
    fn test_mine_pending_transactions() {
        let mut blockchain = Blockchain::new().unwrap();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        blockchain.add_transaction(transaction).unwrap();
        
        let mined_block = blockchain.mine_pending_transactions("Miner").unwrap();
        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(mined_block.index, 1);
        assert_eq!(mined_block.transactions.len(), 1);
        assert_eq!(blockchain.pending_transactions.len(), 1); // Mining reward transaction
    }

    #[test]
    fn test_blockchain_validation() {
        let mut blockchain = Blockchain::new().unwrap();
        assert!(blockchain.validate_chain().unwrap());

        // Add a valid block
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
        assert!(blockchain.validate_chain().unwrap());

        // Tamper with a block
        blockchain.chain[1].transactions[0].amount = 100.0;
        assert!(!blockchain.validate_chain().unwrap());
    }

    #[test]
    fn test_get_balance() {
        let mut blockchain = Blockchain::new().unwrap();
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

        assert_eq!(blockchain.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), -20.0);
        assert_eq!(blockchain.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 20.0);
        assert!(blockchain.get_balance("Miner", &CurrencyType::BasicNeeds).unwrap() > 0.0);
    }

    #[test]
    fn test_adjust_difficulty() {
        let mut blockchain = Blockchain::new().unwrap();
        let initial_difficulty = blockchain.difficulty;

        // Mine 10 blocks quickly
        for _ in 0..10 {
            let transaction = Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 1.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: Utc::now().timestamp(),
                signature: None,
            };
            blockchain.add_transaction(transaction).unwrap();
            blockchain.mine_pending_transactions("Miner").unwrap();
        }

        blockchain.adjust_difficulty();
        assert!(blockchain.difficulty > initial_difficulty);
    }
}