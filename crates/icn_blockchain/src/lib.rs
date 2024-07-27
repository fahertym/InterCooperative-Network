use icn_common::{Block, Transaction, IcnResult, IcnError};
use chrono::Utc;
use sha2::{Sha256, Digest};

pub struct Blockchain {
    pub chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block {
            index: 0,
            timestamp: Utc::now().timestamp(),
            transactions: Vec::new(),
            previous_hash: String::from("0"),
            hash: String::from("genesis_hash"),
        };

        Blockchain {
            chain: vec![genesis_block],
            pending_transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
        self.pending_transactions.push(transaction);
        Ok(())
    }

    pub fn create_block(&mut self) -> IcnResult<Block> {
        let previous_block = self.chain.last()
            .ok_or_else(|| IcnError::Blockchain("Empty blockchain".into()))?;

        let mut new_block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: std::mem::take(&mut self.pending_transactions),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(),
        };

        new_block.hash = self.calculate_hash(&new_block);
        self.chain.push(new_block.clone());
        Ok(new_block)
    }

    fn calculate_hash(&self, block: &Block) -> String {
        let mut hasher = Sha256::new();
        hasher.update(block.index.to_string());
        hasher.update(&block.timestamp.to_string());
        for transaction in &block.transactions {
            hasher.update(&transaction.from);
            hasher.update(&transaction.to);
            hasher.update(&transaction.amount.to_string());
            hasher.update(&transaction.currency_type.to_string());
            hasher.update(&transaction.timestamp.to_string());
        }
        hasher.update(&block.previous_hash);
        format!("{:x}", hasher.finalize())
    }

    pub fn validate_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.previous_hash != previous_block.hash {
                return false;
            }

            if current_block.hash != self.calculate_hash(current_block) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::CurrencyType;

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
        let new_block = blockchain.create_block().unwrap();

        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(new_block.index, 1);
        assert_eq!(new_block.transactions.len(), 1);
        assert_eq!(blockchain.pending_transactions.len(), 0);
    }

    #[test]
    fn test_blockchain_validity() {
        let mut blockchain = Blockchain::new();
        let transaction1 = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 30.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        let transaction2 = Transaction {
            from: "Bob".to_string(),
            to: "Charlie".to_string(),
            amount: 20.0,
            currency_type: CurrencyType::Education,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        blockchain.add_transaction(transaction1).unwrap();
        blockchain.create_block().unwrap();
        blockchain.add_transaction(transaction2).unwrap();
        blockchain.create_block().unwrap();

        assert!(blockchain.validate_chain());

        // Tamper with a block to test invalid chain
        blockchain.chain[1].transactions[0].amount = 100.0;
        assert!(!blockchain.validate_chain());
    }

    #[test]
    fn test_calculate_hash() {
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
        let new_block = blockchain.create_block().unwrap();

        let calculated_hash = blockchain.calculate_hash(&new_block);
        assert_eq!(new_block.hash, calculated_hash);
    }
}