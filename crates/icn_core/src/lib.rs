// icn_core/src/lib.rs

use icn_types::{IcnResult, IcnError, Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType};
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
use log::{info, warn, error};

mod config;
pub use config::Config;

pub struct IcnNode {
    config: Config,
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
    pub fn new(config: Config) -> IcnResult<Self> {
        info!("Initializing ICN Node with configuration: {:?}", config);
        Ok(IcnNode {
            blockchain: Arc::new(Mutex::new(Blockchain::new()?)),
            consensus: Arc::new(Mutex::new(PoCConsensus::new(config.consensus_threshold, config.consensus_quorum)?)),
            currency_system: Arc::new(Mutex::new(CurrencySystem::new())),
            governance_system: Arc::new(Mutex::new(GovernanceSystem::new())),
            identity_manager: Arc::new(Mutex::new(IdentityManager::new())),
            network: Arc::new(Mutex::new(Network::new())),
            sharding_manager: Arc::new(Mutex::new(ShardingManager::new(config.shard_count)?)),
            storage_manager: Arc::new(Mutex::new(StorageManager::new())),
            contract_manager: Arc::new(Mutex::new(ContractManager::new())),
            config,
        })
    }

    pub fn start(&self) -> IcnResult<()> {
        info!("Starting ICN Node components");
        self.network.lock().map_err(|_| IcnError::LockError("Network lock error".into()))?.start()?;
        self.blockchain.lock().map_err(|_| IcnError::LockError("Blockchain lock error".into()))?.start()?;
        self.consensus.lock().map_err(|_| IcnError::LockError("Consensus lock error".into()))?.start()?;
        // Add start methods for other components as needed
        info!("ICN Node started successfully");
        Ok(())
    }

    pub fn stop(&self) -> IcnResult<()> {
        info!("Stopping ICN Node components");
        self.network.lock().map_err(|_| IcnError::LockError("Network lock error".into()))?.stop()?;
        self.blockchain.lock().map_err(|_| IcnError::LockError("Blockchain lock error".into()))?.stop()?;
        self.consensus.lock().map_err(|_| IcnError::LockError("Consensus lock error".into()))?.stop()?;
        // Add stop methods for other components as needed
        info!("ICN Node stopped successfully");
        Ok(())
    }

    pub fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        info!("Processing transaction: {:?}", transaction);
        self.blockchain.lock().map_err(|_| IcnError::LockError("Blockchain lock error".into()))?.add_transaction(transaction)
    }

    pub fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        info!("Creating proposal: {:?}", proposal);
        self.governance_system.lock().map_err(|_| IcnError::LockError("Governance system lock error".into()))?.create_proposal(
            proposal.title,
            proposal.description,
            proposal.proposer,
            proposal.voting_ends_at - proposal.created_at,
            proposal.proposal_type,
            proposal.category,
            proposal.required_quorum,
            proposal.execution_timestamp,
        )
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        info!("Getting balance for address: {} and currency type: {:?}", address, currency_type);
        self.currency_system
            .lock()
            .map_err(|_| IcnError::LockError("Currency system lock error".into()))?
            .get_balance(address, currency_type)
    }

    // Add more methods as needed for interacting with other components
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icn_node_creation() {
        let config = Config::default();
        let node = IcnNode::new(config);
        assert!(node.is_ok());
    }

    #[test]
    fn test_icn_node_start_stop() {
        let config = Config::default();
        let node = IcnNode::new(config).unwrap();
        assert!(node.start().is_ok());
        assert!(node.stop().is_ok());
    }

    // Add more unit tests as needed
}