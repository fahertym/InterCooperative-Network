// File: icn_consensus/src/consensus.rs

use icn_common::{Block, IcnResult, IcnError, Transaction};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use log::{info, warn, error};

#[derive(Debug, Clone, PartialEq)]
pub enum BlockStatus {
    Pending,
    Validated,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct BlockVote {
    pub validator: String,
    pub in_favor: bool,
}

pub struct PoCConsensus {
    threshold: f64,
    quorum: f64,
    validators: HashMap<String, f64>,
    pending_transactions: VecDeque<Transaction>,
    proposed_blocks: HashMap<String, (Block, BlockStatus)>,
    block_votes: HashMap<String, Vec<BlockVote>>,
    broadcast_tx: mpsc::Sender<Block>,
}

impl PoCConsensus {
    pub fn new(threshold: f64, quorum: f64, broadcast_tx: mpsc::Sender<Block>) -> IcnResult<Self> {
        if threshold <= 0.0 || threshold > 1.0 || quorum <= 0.0 || quorum > 1.0 {
            return Err(IcnError::Consensus("Invalid threshold or quorum value".into()));
        }

        Ok(PoCConsensus {
            threshold,
            quorum,
            validators: HashMap::new(),
            pending_transactions: VecDeque::new(),
            proposed_blocks: HashMap::new(),
            block_votes: HashMap::new(),
            broadcast_tx,
        })
    }

    pub fn add_validator(&mut self, validator_id: String, initial_reputation: f64) -> IcnResult<()> {
        if initial_reputation < 0.0 || initial_reputation > 1.0 {
            return Err(IcnError::Consensus("Invalid initial reputation".into()));
        }
        self.validators.insert(validator_id, initial_reputation);
        Ok(())
    }

    pub fn add_pending_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push_back(transaction);
    }

    pub async fn propose_block(&mut self, proposer: String) -> IcnResult<String> {
        let transactions: Vec<Transaction> = self.pending_transactions.drain(..100.min(self.pending_transactions.len())).collect();
        
        if transactions.is_empty() {
            return Err(IcnError::Consensus("No pending transactions to create a block".into()));
        }

        let previous_block = self.get_latest_block()?;
        let new_block = Block::new(
            previous_block.index + 1,
            transactions,
            previous_block.hash.clone(),
        );

        let block_hash = new_block.hash.clone();
        self.proposed_blocks.insert(block_hash.clone(), (new_block.clone(), BlockStatus::Pending));
        self.block_votes.insert(block_hash.clone(), Vec::new());

        // Broadcast the proposed block to other nodes
        if let Err(e) = self.broadcast_tx.send(new_block).await {
            error!("Failed to broadcast proposed block: {}", e);
            return Err(IcnError::Consensus("Failed to broadcast proposed block".into()));
        }

        info!("Block proposed by {}: {}", proposer, block_hash);
        Ok(block_hash)
    }

    pub fn validate_proposed_block(&mut self, block_hash: &str, validator: String) -> IcnResult<bool> {
        let (block, status) = self.proposed_blocks.get(block_hash)
            .ok_or_else(|| IcnError::Consensus("Proposed block not found".into()))?;

        if *status != BlockStatus::Pending {
            return Err(IcnError::Consensus("Block is not in pending state".into()));
        }

        if !self.validate_block(block)? {
            self.proposed_blocks.get_mut(block_hash).unwrap().1 = BlockStatus::Rejected;
            return Ok(false);
        }

        let validator_reputation = *self.validators.get(&validator)
            .ok_or_else(|| IcnError::Consensus("Validator not found".into()))?;

        let vote = BlockVote {
            validator: validator.clone(),
            in_favor: true,
        };

        self.block_votes.get_mut(block_hash).unwrap().push(vote);

        let total_votes: f64 = self.block_votes[block_hash].iter()
            .map(|v| self.validators[&v.validator])
            .sum();

        let votes_in_favor: f64 = self.block_votes[block_hash].iter()
            .filter(|v| v.in_favor)
            .map(|v| self.validators[&v.validator])
            .sum();

        let total_reputation: f64 = self.validators.values().sum();

        if total_votes / total_reputation >= self.quorum {
            if votes_in_favor / total_votes >= self.threshold {
                self.proposed_blocks.get_mut(block_hash).unwrap().1 = BlockStatus::Validated;
                info!("Block {} validated", block_hash);
                Ok(true)
            } else {
                self.proposed_blocks.get_mut(block_hash).unwrap().1 = BlockStatus::Rejected;
                info!("Block {} rejected", block_hash);
                Ok(false)
            }
        } else {
            Ok(true) // Block is still pending
        }
    }

    pub fn validate_block(&self, block: &Block) -> IcnResult<bool> {
        // Check block index
        let previous_block = self.get_latest_block()?;
        if block.index != previous_block.index + 1 {
            return Ok(false);
        }

        // Check previous hash
        if block.previous_hash != previous_block.hash {
            return Ok(false);
        }

        // Verify block hash
        if block.hash != block.calculate_hash() {
            return Ok(false);
        }

        // Check timestamp
        if block.timestamp <= previous_block.timestamp {
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
        // Implement transaction validation logic here
        // For example, check if the sender has sufficient balance, if the transaction is properly signed, etc.
        // This is a placeholder implementation
        Ok(true)
    }

    fn get_latest_block(&self) -> IcnResult<Block> {
        // This method should return the latest validated block from the blockchain
        // For now, we'll return a placeholder block
        Ok(Block {
            index: 0,
            timestamp: 0,
            transactions: Vec::new(),
            previous_hash: "genesis".to_string(),
            hash: "genesis".to_string(),
        })
    }

    pub fn get_total_reputation(&self) -> f64 {
        self.validators.values().sum()
    }

    pub fn is_quorum_reached(&self, participating_reputation: f64) -> bool {
        let total_reputation = self.get_total_reputation();
        participating_reputation / total_reputation >= self.quorum
    }

    pub fn is_consensus_reached(&self, approving_reputation: f64, participating_reputation: f64) -> bool {
        approving_reputation / participating_reputation >= self.threshold
    }

    pub fn update_reputation(&mut self, validator_id: &str, change: f64) -> IcnResult<()> {
        let reputation = self.validators.get_mut(validator_id)
            .ok_or_else(|| IcnError::Consensus("Validator not found".into()))?;
        *reputation += change;
        *reputation = reputation.clamp(0.0, 1.0);
        Ok(())
    }

    pub fn get_pending_blocks(&self) -> Vec<String> {
        self.proposed_blocks.iter()
            .filter(|(_, (_, status))| *status == BlockStatus::Pending)
            .map(|(hash, _)| hash.clone())
            .collect()
    }

    pub fn get_validated_blocks(&self) -> Vec<String> {
        self.proposed_blocks.iter()
            .filter(|(_, (_, status))| *status == BlockStatus::Validated)
            .map(|(hash, _)| hash.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    async fn create_test_consensus() -> PoCConsensus {
        let (tx, _rx) = mpsc::channel(100);
        PoCConsensus::new(0.66, 0.51, tx).unwrap()
    }

    #[tokio::test]
    async fn test_consensus_creation() {
        let consensus = create_test_consensus().await;
        assert_eq!(consensus.threshold, 0.66);
        assert_eq!(consensus.quorum, 0.51);
    }

    #[tokio::test]
    async fn test_add_validator() {
        let mut consensus = create_test_consensus().await;
        assert!(consensus.add_validator("validator1".to_string(), 0.8).is_ok());
        assert!(consensus.add_validator("validator2".to_string(), 0.7).is_ok());
        assert_eq!(consensus.validators.len(), 2);
    }

    #[tokio::test]
    async fn test_propose_and_validate_block() {
        let mut consensus = create_test_consensus().await;
        consensus.add_validator("validator1".to_string(), 0.8).unwrap();
        consensus.add_validator("validator2".to_string(), 0.7).unwrap();

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: icn_common::CurrencyType::BasicNeeds,
            timestamp: 12345,
            signature: None,
        };
        consensus.add_pending_transaction(transaction);

        let block_hash = consensus.propose_block("validator1".to_string()).await.unwrap();
        assert!(consensus.validate_proposed_block(&block_hash, "validator1".to_string()).unwrap());
        assert!(consensus.validate_proposed_block(&block_hash, "validator2".to_string()).unwrap());

        let validated_blocks = consensus.get_validated_blocks();
        assert_eq!(validated_blocks.len(), 1);
        assert_eq!(validated_blocks[0], block_hash);
    }

    #[tokio::test]
    async fn test_reputation_update() {
        let mut consensus = create_test_consensus().await;
        consensus.add_validator("validator1".to_string(), 0.5).unwrap();
        
        consensus.update_reputation("validator1", 0.2).unwrap();
        assert_eq!(consensus.validators["validator1"], 0.7);

        consensus.update_reputation("validator1", 0.4).unwrap();
        assert_eq!(consensus.validators["validator1"], 1.0);

        consensus.update_reputation("validator1", -1.5).unwrap();
        assert_eq!(consensus.validators["validator1"], 0.0);
    }
}