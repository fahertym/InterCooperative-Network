// crates/icn_core/src/lib.rs

use icn_types::{IcnResult, IcnError, Block, Transaction, CurrencyType};
use std::sync::{Arc, RwLock};
use chrono::Utc;

mod blockchain;
mod consensus;
mod currency;
mod governance;

pub use blockchain::Blockchain;
pub use consensus::Consensus;
pub use currency::CurrencySystem;
pub use governance::GovernanceSystem;

pub struct IcnNode {
    blockchain: Arc<RwLock<Blockchain>>,
    consensus: Arc<RwLock<Consensus>>,
    currency_system: Arc<RwLock<CurrencySystem>>,
    governance_system: Arc<RwLock<GovernanceSystem>>,
}

impl IcnNode {
    pub fn new() -> Self {
        IcnNode {
            blockchain: Arc::new(RwLock::new(Blockchain::new())),
            consensus: Arc::new(RwLock::new(Consensus::new())),
            currency_system: Arc::new(RwLock::new(CurrencySystem::new())),
            governance_system: Arc::new(RwLock::new(GovernanceSystem::new())),
        }
    }

    pub fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        let mut blockchain = self.blockchain.write().map_err(|_| IcnError::General("Failed to acquire blockchain write lock".to_string()))?;
        blockchain.add_transaction(transaction)
    }

    pub fn create_block(&self) -> IcnResult<Block> {
        let mut blockchain = self.blockchain.write().map_err(|_| IcnError::General("Failed to acquire blockchain write lock".to_string()))?;
        let consensus = self.consensus.read().map_err(|_| IcnError::General("Failed to acquire consensus read lock".to_string()))?;
        
        let new_block = blockchain.create_block()?;
        
        if consensus.validate_block(&new_block) {
            blockchain.add_block(new_block.clone())?;
            Ok(new_block)
        } else {
            Err(IcnError::Consensus("Block validation failed".to_string()))
        }
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let currency_system = self.currency_system.read().map_err(|_| IcnError::General("Failed to acquire currency system read lock".to_string()))?;
        currency_system.get_balance(address, currency_type)
    }

    pub fn create_proposal(&self, title: String, description: String, proposer: String) -> IcnResult<String> {
        let mut governance_system = self.governance_system.write().map_err(|_| IcnError::General("Failed to acquire governance system write lock".to_string()))?;
        governance_system.create_proposal(title, description, proposer)
    }

    pub fn vote_on_proposal(&self, proposal_id: &str, voter: &str, vote: bool) -> IcnResult<()> {
        let mut governance_system = self.governance_system.write().map_err(|_| IcnError::General("Failed to acquire governance system write lock".to_string()))?;
        governance_system.vote_on_proposal(proposal_id, voter, vote)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let node = IcnNode::new();
        assert_eq!(node.blockchain.read().unwrap().chain().len(), 1);
    }

    #[test]
    fn test_process_transaction() {
        let node = IcnNode::new();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        assert!(node.process_transaction(transaction).is_ok());
        assert_eq!(node.blockchain.read().unwrap().pending_transactions().len(), 1);
    }

    #[test]
    fn test_create_block() {
        let node = IcnNode::new();
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        node.process_transaction(transaction).unwrap();
        let block = node.create_block().unwrap();
        assert_eq!(block.index, 1);
        assert_eq!(node.blockchain.read().unwrap().chain().len(), 2);
    }
}