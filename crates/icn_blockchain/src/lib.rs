use icn_common::{Block, Transaction, IcnResult, IcnError, Hashable, CurrencyType};
use std::collections::VecDeque;
use chrono::Utc;
use log::info;

const MAX_TRANSACTIONS_PER_BLOCK: usize = 100;

pub struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: VecDeque<Transaction>,
}

impl Blockchain {
    pub fn new() -> IcnResult<Self> {
        Ok(Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: VecDeque::new(),
        })
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
        if self.verify_transaction(&transaction)? {
            self.pending_transactions.push_back(transaction);
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid transaction".into()))
        }
    }

    pub fn create_block(&mut self) -> IcnResult<Block> {
        let previous_block = self.chain.last()
            .ok_or_else(|| IcnError::Blockchain("No previous block found".into()))?;
        
        let transactions: Vec<Transaction> = self.pending_transactions
            .drain(..std::cmp::min(self.pending_transactions.len(), MAX_TRANSACTIONS_PER_BLOCK))
            .collect();

        let new_block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions,
            previous_hash: previous_block.hash.clone(),
            hash: String::new(), // Will be set in calculate_hash
        };

        let new_block = self.calculate_hash(new_block);
        self.chain.push(new_block.clone());
        Ok(new_block)
    }

    fn calculate_hash(&self, mut block: Block) -> Block {
        block.hash = block.hash();
        block
    }

    pub fn verify_transaction(&self, transaction: &Transaction) -> IcnResult<bool> {
        // Implement transaction verification logic
        // For example, check if the sender has sufficient balance
        // and if the signature is valid
        Ok(true) // Placeholder implementation
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn get_block_by_index(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        self.chain.iter().find(|block| block.hash == hash)
    }

    pub fn validate_chain(&self) -> IcnResult<bool> {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != current_block.hash() {
                return Ok(false);
            }

            if current_block.previous_hash != previous_block.hash {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn start(&self) -> IcnResult<()> {
        info!("Blockchain started");
        Ok(())
    }

    pub fn stop(&self) -> IcnResult<()> {
        info!("Blockchain stopped");
        Ok(())
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
    fn test_add_transaction_and_create_block() {
        let mut blockchain = Blockchain::new().unwrap();
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            Utc::now().timestamp(),
        );

        assert!(blockchain.add_transaction(transaction).is_ok());
        assert_eq!(blockchain.pending_transactions.len(), 1);

        let new_block = blockchain.create_block().unwrap();
        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(new_block.index, 1);
        assert_eq!(new_block.transactions.len(), 1);
    }

    #[test]
    fn test_blockchain_validation() {
        let mut blockchain = Blockchain::new().unwrap();
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            Utc::now().timestamp(),
        );

        blockchain.add_transaction(transaction).unwrap();
        blockchain.create_block().unwrap();

        assert!(blockchain.validate_chain().unwrap());

        // Tamper with a block
        blockchain.chain[1].transactions[0].amount = 200.0;
        assert!(!blockchain.validate_chain().unwrap());
    }

    #[test]
    fn test_get_block_methods() {
        let mut blockchain = Blockchain::new().unwrap();
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            Utc::now().timestamp(),
        );

        blockchain.add_transaction(transaction).unwrap();
        let new_block = blockchain.create_block().unwrap();

        assert_eq!(blockchain.get_latest_block().unwrap().hash, new_block.hash);
        assert_eq!(blockchain.get_block_by_index(1).unwrap().hash, new_block.hash);
        assert_eq!(blockchain.get_block_by_hash(&new_block.hash).unwrap().index, 1);
    }
}