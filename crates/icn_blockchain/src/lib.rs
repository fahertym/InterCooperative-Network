// File: icn_blockchain/src/lib.rs

use chrono::Utc;
use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
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
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(&self.timestamp.to_string());
        hasher.update(serde_json::to_string(&self.transactions).unwrap());
        hasher.update(&self.previous_hash);
        hasher.update(&self.nonce.to_string());
        format!("{:x}", hasher.finalize())
    }

    pub fn mine(&mut self, difficulty: usize) {
        let target = vec![0; difficulty];
        while &self.hash[..difficulty] != &target[..] {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
    balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            difficulty,
            balances: HashMap::new(),
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

        let new_block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            &self.get_latest_block().hash,
        );
        self.add_block(new_block)?;

        self.pending_transactions.clear();
        Ok(())
    }

    fn add_block(&mut self, mut block: Block) -> IcnResult<()> {
        block.mine(self.difficulty);
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
        }
        true
    }

    fn validate_transaction(&self, transaction: &Transaction) -> IcnResult<bool> {
        if transaction.from == "Network" {
            return Ok(true); // Allow mining rewards
        }

        let sender_balance = self.get_balance(&transaction.from, &transaction.currency_type)?;
        if sender_balance < transaction.amount {
            return Ok(false);
        }

        // Additional validation logic can be added here (e.g., signature verification)

        Ok(true)
    }

    fn update_balances(&mut self) -> IcnResult<()> {
        for transaction in &self.get_latest_block().transactions {
            let from_balance = self.balances
                .entry(transaction.from.clone())
                .or_insert_with(HashMap::new)
                .entry(transaction.currency_type.clone())
                .or_insert(0.0);
            *from_balance -= transaction.amount;

            let to_balance = self.balances
                .entry(transaction.to.clone())
                .or_insert_with(HashMap::new)
                .entry(transaction.currency_type.clone())
                .or_insert(0.0);
            *to_balance += transaction.amount;
        }
        Ok(())
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        Ok(*self.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .unwrap_or(&0.0))
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
        blockchain.balances.entry("Alice".to_string())
            .or_insert_with(HashMap::new)
            .insert(CurrencyType::BasicNeeds, 100.0);

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

        blockchain.balances.entry("Alice".to_string())
            .or_insert_with(HashMap::new)
            .insert(CurrencyType::BasicNeeds, 100.0);

        assert!(blockchain.add_transaction(transaction).is_ok());
        assert!(blockchain.mine_pending_transactions("Miner").is_ok());

        assert!(blockchain.is_chain_valid());

        // Tamper with a block to test invalid chain
        blockchain.chain[1].transactions[0].amount = 100.0;
        assert!(!blockchain.is_chain_valid());
    }

    #[test]
    fn test_insufficient_balance() {
        let mut blockchain = Blockchain::new(2);
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        // Alice has no balance
        assert!(blockchain.add_transaction(transaction).is_err());
    }
}