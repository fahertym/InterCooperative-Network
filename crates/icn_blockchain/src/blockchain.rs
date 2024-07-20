// crates/icn_core/src/blockchain.rs

use icn_common_types::{IcnResult, IcnError, Block, Transaction};
use chrono::Utc;
use sha2::{Sha256, Digest};

pub struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> IcnResult<Self> {
        Ok(Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: Vec::new(),
        })
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
        if self.validate_transaction(&transaction) {
            self.pending_transactions.push(transaction);
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid transaction".to_string()))
        }
    }

    pub fn create_block(&mut self) -> IcnResult<Block> {
        let previous_block = self.chain.last()
            .ok_or_else(|| IcnError::Blockchain("No previous block found".to_string()))?;
        
        let new_block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: self.pending_transactions.clone(),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(), // Will be set in calculate_hash
        };

        let new_block = self.calculate_hash(new_block);
        Ok(new_block)
    }

    pub fn add_block(&mut self, block: Block) -> IcnResult<()> {
        if self.validate_block(&block) {
            self.chain.push(block);
            self.pending_transactions.clear();
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid block".to_string()))
        }
    }

    fn calculate_hash(&self, mut block: Block) -> Block {
        let mut hasher = Sha256::new();
        hasher.update(block.index.to_string());
        hasher.update(block.timestamp.to_string());
        for transaction in &block.transactions {
            hasher.update(transaction.from.as_bytes());
            hasher.update(transaction.to.as_bytes());
            hasher.update(transaction.amount.to_string());
            hasher.update(format!("{:?}", transaction.currency_type));
        }
        hasher.update(&block.previous_hash);
        
        let hash = format!("{:x}", hasher.finalize());
        block.hash = hash;
        block
    }

    fn validate_transaction(&self, transaction: &Transaction) -> bool {
        // Implement transaction validation logic
        // For example, check if the sender has sufficient balance
        true // Placeholder
    }

    fn validate_block(&self, block: &Block) -> bool {
        if block.index != self.chain.len() as u64 {
            return false;
        }

        if let Some(last_block) = self.chain.last() {
            if block.previous_hash != last_block.hash {
                return false;
            }
        } else if block.index != 0 {
            return false;
        }

        let calculated_hash = self.calculate_hash(block.clone()).hash;
        calculated_hash == block.hash
    }

    pub fn chain(&self) -> &[Block] {
        &self.chain
    }

    pub fn pending_transactions(&self) -> &[Transaction] {
        &self.pending_transactions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_types::CurrencyType;

    #[test]
    fn test_new_blockchain() {
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
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        assert!(blockchain.add_transaction(transaction).is_ok());
        assert_eq!(blockchain.pending_transactions.len(), 1);
    }

    #[test]
    fn test_create_and_add_block() {
        let mut blockchain = Blockchain::new().unwrap();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        blockchain.add_transaction(transaction).unwrap();
        
        let new_block = blockchain.create_block().unwrap();
        assert_eq!(new_block.index, 1);
        assert_eq!(new_block.transactions.len(), 1);
        
        assert!(blockchain.add_block(new_block).is_ok());
        assert_eq!(blockchain.chain.len(), 2);
        assert!(blockchain.pending_transactions.is_empty());
    }
}
