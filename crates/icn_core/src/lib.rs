use icn_types::{IcnResult, IcnError};
use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::GovernanceSystem;
use icn_identity::IdentityManager;
use icn_network::Network;
use icn_sharding::ShardingManager;
use icn_storage::StorageManager;
use icn_vm::ContractManager;

use std::sync::{Arc, Mutex};

pub struct IcnNode {
    blockchain: Arc<Mutex<Blockchain>>,
    consensus: Arc<Mutex<PoCConsensus>>,
    currency_system: Arc<Mutex<CurrencySystem>>,
    governance_system: Arc<Mutex<GovernanceSystem>>,
    identity_manager: Arc<Mutex<IdentityManager>>,
    network: Arc<Mutex<Network>>,
    sharding_manager: Arc<Mutex<ShardingManager>>,
    storage_manager: Arc<Mutex<StorageManager>>,
    contract_manager: Arc<Mutex<ContractManager>>,
}

impl IcnNode {
    pub fn new() -> IcnResult<Self> {
        Ok(IcnNode {
            blockchain: Arc::new(Mutex::new(Blockchain::new()?)),
            consensus: Arc::new(Mutex::new(PoCConsensus::new(0.66, 0.51)?)),
            currency_system: Arc::new(Mutex::new(CurrencySystem::new())),
            governance_system: Arc::new(Mutex::new(GovernanceSystem::new())),
            identity_manager: Arc::new(Mutex::new(IdentityManager::new())),
            network: Arc::new(Mutex::new(Network::new())),
            sharding_manager: Arc::new(Mutex::new(ShardingManager::new(4)?)),
            storage_manager: Arc::new(Mutex::new(StorageManager::new())),
            contract_manager: Arc::new(Mutex::new(ContractManager::new())),
        })
    }

    pub fn start(&self) -> IcnResult<()> {
        // Initialize and start all components
        self.network.lock().map_err(|_| IcnError::LockError)?.start()?;
        self.blockchain.lock().map_err(|_| IcnError::LockError)?.start()?;
        self.consensus.lock().map_err(|_| IcnError::LockError)?.start()?;
        // Add more component initializations as needed

        Ok(())
    }

    pub fn stop(&self) -> IcnResult<()> {
        // Stop all components
        self.network.lock().map_err(|_| IcnError::LockError)?.stop()?;
        self.blockchain.lock().map_err(|_| IcnError::LockError)?.stop()?;
        self.consensus.lock().map_err(|_| IcnError::LockError)?.stop()?;
        // Add more component stop calls as needed

        Ok(())
    }

    // Add more methods to interact with different components
    pub fn process_transaction(&self, transaction: icn_types::Transaction) -> IcnResult<()> {
        self.blockchain.lock().map_err(|_| IcnError::LockError)?.add_transaction(transaction)
    }

    pub fn create_proposal(&self, proposal: icn_types::Proposal) -> IcnResult<String> {
        self.governance_system.lock().map_err(|_| IcnError::LockError)?.create_proposal(
            proposal.title,
            proposal.description,
            proposal.proposer,
            proposal.voting_period,
            proposal.proposal_type,
            proposal.category,
            proposal.required_quorum,
            proposal.execution_timestamp,
        )
    }

    // Add more methods as needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icn_node_creation() {
        let node = IcnNode::new();
        assert!(node.is_ok());
    }

    #[test]
    fn test_icn_node_start_stop() {
        let node = IcnNode::new().unwrap();
        assert!(node.start().is_ok());
        assert!(node.stop().is_ok());
    }

    // Add more tests as needed
}