// File: icn_consensus/src/lib.rs

use icn_blockchain::Block;
use icn_common::{IcnResult, IcnError, Transaction};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, error};

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
            blockchain: Arc::new(RwLock::new(Vec::new())),
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

            for (validator, reputation) in &self.validators {
                if self.validate_block(block)? {
                    votes_for += reputation;
                }
                total_votes += reputation;

                if total_votes >= quorum_reputation {
                    if votes_for / total_votes >= self.threshold {
                        blocks_to_add.push(block.clone());
                    } else {
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
        // Implement block validation logic here
        // For simplicity, we'll assume all blocks are valid
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::CurrencyType;

    fn create_test_block(index: u64, previous_hash: &str) -> Block {
        Block::new(
            index,
            vec![Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 100.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: 0,
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
        assert_eq!(blockchain.len(), 1);
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
        assert_eq!(blockchain.len(), 1);
    }
}