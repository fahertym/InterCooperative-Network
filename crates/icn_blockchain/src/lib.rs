use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use crate::transaction::Transaction;

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
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
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

    pub fn genesis() -> Self {
        Block::new(0, vec![], String::from("0"))
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
