use crate::Transaction;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub gas_used: u64,
    pub smart_contract_results: HashMap<String, String>,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            gas_used: 0,
            smart_contract_results: HashMap::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(self.timestamp.to_string());
        for transaction in &self.transactions {
            hasher.update(transaction.to_string());
        }
        hasher.update(&self.previous_hash);
        hasher.update(self.nonce.to_string());
        format!("{:x}", hasher.finalize())
    }

    pub fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }

    pub fn add_smart_contract_result(&mut self, contract_id: String, result: String) {
        self.smart_contract_results.insert(contract_id, result);
    }

    pub fn get_smart_contract_result(&self, contract_id: &str) -> Option<&String> {
        self.smart_contract_results.get(contract_id)
    }

    pub fn set_gas_used(&mut self, gas: u64) {
        self.gas_used = gas;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::CurrencyType;

    #[test]
    fn test_new_block() {
        let transactions = vec![Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        )];
        let block = Block::new(1, transactions, "previous_hash".to_string());
        assert_eq!(block.index, 1);
        assert_eq!(block.previous_hash, "previous_hash");
        assert!(!block.hash.is_empty());
    }

    #[test]
    fn test_mine_block() {
        let transactions = vec![Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        )];
        let mut block = Block::new(1, transactions, "previous_hash".to_string());
        block.mine_block(2);
        assert!(block.hash.starts_with("00"));
    }

    #[test]
    fn test_smart_contract_results() {
        let mut block = Block::new(1, vec![], "previous_hash".to_string());
        block.add_smart_contract_result("contract1".to_string(), "result1".to_string());
        assert_eq!(block.get_smart_contract_result("contract1"), Some(&"result1".to_string()));
        assert_eq!(block.get_smart_contract_result("nonexistent"), None);
    }
}