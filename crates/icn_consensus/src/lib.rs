// File: icn_consensus/src/lib.rs

use icn_blockchain::Block;
use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, warn, error};
use chrono::Utc;

pub struct PoCConsensus {
    threshold: f64,
    quorum: f64,
    validators: HashMap<String, f64>,
    pending_blocks: Vec<Block>,
    blockchain: Arc<RwLock<Vec<Block>>>,
}

impl PoCConsensus {
    pub fn new(threshold: f64, quorum: f64) -> IcnResult<Self> {
        if !(0.0..=1.0).contains(&threshold) || !(0.0..=1.0).contains(&quorum) {
            return Err(IcnError::Consensus("Invalid threshold or quorum value".into()));
        }

        Ok(PoCConsensus {
            threshold,
            quorum,
            validators: HashMap::new(),
            pending_blocks: Vec::new(),
            blockchain: Arc::new(RwLock::new(vec![Block::new(0, Vec::new(), String::from("0"), 4)])),
        })
    }

    pub fn start(&self) -> IcnResult<()> {
        info!("PoC Consensus mechanism started");
        Ok(())
    }

    pub fn stop(&self) -> IcnResult<()> {
        info!("PoC Consensus mechanism stopped");
        Ok(())
    }

    pub fn add_validator(&mut self, id: String, reputation: f64) -> IcnResult<()> {
        if !(0.0..=1.0).contains(&reputation) {
            return Err(IcnError::Consensus("Invalid initial reputation".into()));
        }
        self.validators.insert(id, reputation);
        Ok(())
    }

    pub fn process_new_block(&mut self, block: Block) -> IcnResult<()> {
        self.pending_blocks.push(block);
        self.try_reach_consensus()
    }

    fn try_reach_consensus(&mut self) -> IcnResult<()> {
        let total_reputation: f64 = self.validators.values().sum();
        let quorum_reputation = total_reputation * self.quorum;

        let mut blocks_to_add = Vec::new();

        for block in &self.pending_blocks {
            let mut votes_for = 0.0;
            let mut total_votes = 0.0;

            for (_, reputation) in &self.validators {
                if self.validate_block(block)? {
                    votes_for += reputation;
                }
                total_votes += reputation;

                if total_votes >= quorum_reputation {
                    if votes_for / total_votes >= self.threshold {
                        blocks_to_add.push(block.clone());
                    } else {
                        warn!("Block rejected by consensus: {:?}", block);
                        return Err(IcnError::Consensus("Block rejected by consensus".into()));
                    }
                }
            }
        }

        for block in blocks_to_add {
            self.add_block_to_chain(block)?;
        }

        self.pending_blocks.retain(|b| !blocks_to_add.contains(b));

        Ok(())
    }

    fn validate_block(&self, block: &Block) -> IcnResult<bool> {
        // Check if the block's previous hash matches the last block in the chain
        let blockchain = self.blockchain.read().map_err(|_| IcnError::Consensus("Failed to read blockchain".into()))?;
        let last_block = blockchain.last().ok_or_else(|| IcnError::Consensus("Blockchain is empty".into()))?;
        
        if block.previous_hash != last_block.hash {
            return Ok(false);
        }

        // Verify block hash
        if block.hash != block.calculate_hash() {
            return Ok(false);
        }

        // Validate transactions
        for transaction in &block.transactions {
            if !self.validate_transaction(transaction)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn validate_transaction(&self, transaction: &Transaction) -> IcnResult<bool> {
        // Check if the transaction amount is positive
        if transaction.amount <= 0.0 {
            return Ok(false);
        }

        // Check if the sender has sufficient balance
        let blockchain = self.blockchain.read().map_err(|_| IcnError::Consensus("Failed to read blockchain".into()))?;
        let sender_balance = self.get_balance(&blockchain, &transaction.from, &transaction.currency_type);
        
        if sender_balance < transaction.amount {
            return Ok(false);
        }

        // Add more transaction validation rules as needed

        Ok(true)
    }

    fn add_block_to_chain(&mut self, block: Block) -> IcnResult<()> {
        let mut blockchain = self.blockchain.write().map_err(|_| IcnError::Consensus("Failed to write to blockchain".into()))?;
        blockchain.push(block);
        Ok(())
    }

    pub fn get_blockchain(&self) -> IcnResult<Vec<Block>> {
        let blockchain = self.blockchain.read().map_err(|_| IcnError::Consensus("Failed to read blockchain".into()))?;
        Ok(blockchain.clone())
    }

    fn get_balance(&self, blockchain: &[Block], address: &str, currency_type: &CurrencyType) -> f64 {
        let mut balance = 0.0;
        for block in blockchain {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block(index: u64, previous_hash: &str) -> Block {
        Block::new(
            index,
            vec![Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 100.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: Utc::now().timestamp(),
                signature: None,
            }],
            previous_hash.to_string(),
            1,
        )
    }

    #[test]
    fn test_poc_consensus_creation() {
        let consensus = PoCConsensus::new(0.66, 0.51);
        assert!(consensus.is_ok());
    }

    #[test]
    fn test_add_validator() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        assert!(consensus.add_validator("validator1".to_string(), 0.8).is_ok());
        assert!(consensus.add_validator("validator2".to_string(), 0.7).is_ok());
        assert_eq!(consensus.validators.len(), 2);
    }

    #[test]
    fn test_process_new_block() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_validator("validator1".to_string(), 0.8).unwrap();
        consensus.add_validator("validator2".to_string(), 0.7).unwrap();

        let new_block = create_test_block(1, "test_hash_0");
        assert!(consensus.process_new_block(new_block).is_ok());

        let blockchain = consensus.get_blockchain().unwrap();
        assert_eq!(blockchain.len(), 2);  // Genesis block + 1 new block
    }

    #[test]
    fn test_consensus_threshold() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_validator("validator1".to_string(), 0.8).unwrap();
        consensus.add_validator("validator2".to_string(), 0.7).unwrap();
        consensus.add_validator("validator3".to_string(), 0.6).unwrap();

        let new_block = create_test_block(1, "test_hash_0");
        assert!(consensus.process_new_block(new_block).is_ok());

        // The total reputation is 2.1, and the quorum is 0.51 * 2.1 = 1.071
        // The threshold is 0.66 * 1.071 = 0.70686
        // So if validators with total reputation > 0.70686 approve, the block should be added

        let blockchain = consensus.get_blockchain().unwrap();
        assert_eq!(blockchain.len(), 2);
    }

    #[test]
    fn test_invalid_block() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_validator("validator1".to_string(), 0.8).unwrap();
        consensus.add_validator("validator2".to_string(), 0.7).unwrap();

        // Create an invalid block with incorrect previous_hash
        let mut invalid_block = create_test_block(1, "invalid_previous_hash");
        invalid_block.hash = invalid_block.calculate_hash();

        assert!(consensus.process_new_block(invalid_block).is_err());

        let blockchain = consensus.get_blockchain().unwrap();
        assert_eq!(blockchain.len(), 1);  // Only genesis block should remain
    }

    #[test]
    fn test_insufficient_balance_transaction() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_validator("validator1".to_string(), 0.8).unwrap();
        consensus.add_validator("validator2".to_string(), 0.7).unwrap();

        // Create a block with a transaction that has insufficient balance
        let invalid_transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 1000.0,  // Assume Alice doesn't have this much balance
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        let mut invalid_block = Block::new(1, vec![invalid_transaction], "test_hash_0".to_string(), 1);
        invalid_block.hash = invalid_block.calculate_hash();

        assert!(consensus.process_new_block(invalid_block).is_err());

        let blockchain = consensus.get_blockchain().unwrap();
        assert_eq!(blockchain.len(), 1);  // Only genesis block should remain
    }

    #[test]
    fn test_multiple_blocks() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_validator("validator1".to_string(), 0.8).unwrap();
        consensus.add_validator("validator2".to_string(), 0.7).unwrap();

        let block1 = create_test_block(1, "test_hash_0");
        let block2 = create_test_block(2, &block1.hash);
        let block3 = create_test_block(3, &block2.hash);

        assert!(consensus.process_new_block(block1).is_ok());
        assert!(consensus.process_new_block(block2).is_ok());
        assert!(consensus.process_new_block(block3).is_ok());

        let blockchain = consensus.get_blockchain().unwrap();
        assert_eq!(blockchain.len(), 4);  // Genesis block + 3 new blocks
    }
}