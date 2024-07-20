use icn_common_types::{IcnResult, IcnError, Block, Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType};
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::GovernanceSystem;
use icn_identity::IdentityManager;
use icn_network::Network;
use icn_sharding::ShardingManager;
use icn_vm::ContractManager;

use std::sync::{Arc, Mutex};
use log::{info, warn, error};

pub struct Config {
    pub shard_count: u64,
    pub consensus_threshold: f64,
    pub consensus_quorum: f64,
    pub network_port: u16,
}

pub struct IcnNode {
    config: Config,
    blockchain: Arc<Mutex<Blockchain>>,
    consensus: Arc<Mutex<PoCConsensus>>,
    currency_system: Arc<Mutex<CurrencySystem>>,
    governance_system: Arc<Mutex<GovernanceSystem>>,
    identity_manager: Arc<Mutex<IdentityManager>>,
    network: Arc<Mutex<Network>>,
    sharding_manager: Arc<Mutex<ShardingManager>>,
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
            network: Arc::new(Mutex::new(Network::new(config.network_port))),
            sharding_manager: Arc::new(Mutex::new(ShardingManager::new(config.shard_count)?)),
            contract_manager: Arc::new(Mutex::new(ContractManager::new())),
            config,
        })
    }

    pub fn start(&self) -> IcnResult<()> {
        info!("Starting ICN Node components");
        self.network.lock().map_err(|e| IcnError::Blockchain(e.to_string()))?.start()?;
        self.blockchain.lock().map_err(|e| IcnError::Blockchain(e.to_string()))?.start()?;
        self.consensus.lock().map_err(|e| IcnError::Consensus(e.to_string()))?.start()?;
        // Add start methods for other components as needed
        info!("ICN Node started successfully");
        Ok(())
    }

    pub fn stop(&self) -> IcnResult<()> {
        info!("Stopping ICN Node components");
        self.network.lock().map_err(|e| IcnError::Network(e.to_string()))?.stop()?;
        self.blockchain.lock().map_err(|e| IcnError::Blockchain(e.to_string()))?.stop()?;
        self.consensus.lock().map_err(|e| IcnError::Consensus(e.to_string()))?.stop()?;
        // Add stop methods for other components as needed
        info!("ICN Node stopped successfully");
        Ok(())
    }

    pub fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        info!("Processing transaction: {:?}", transaction);
        let mut blockchain = self.blockchain.lock().map_err(|e| IcnError::Blockchain(e.to_string()))?;
        let mut currency_system = self.currency_system.lock().map_err(|e| IcnError::Currency(e.to_string()))?;

        if !self.verify_transaction(&transaction)? {
            return Err(IcnError::Blockchain("Transaction verification failed".into()));
        }

        currency_system.transfer(
            &transaction.from,
            &transaction.to,
            transaction.amount,
            &transaction.currency_type,
        )?;

        blockchain.add_transaction(transaction)?;

        Ok(())
    }

    fn verify_transaction(&self, transaction: &Transaction) -> IcnResult<bool> {
        let sender_balance = self.get_balance(&transaction.from, &transaction.currency_type)?;
        if sender_balance < transaction.amount {
            return Ok(false);
        }

        // Implement signature verification
        let identity_manager = self.identity_manager.lock().map_err(|e| IcnError::Identity(e.to_string()))?;
        let public_key = identity_manager.get_public_key(&transaction.from)?;
        transaction.verify(&public_key)
    }

    pub fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        info!("Creating proposal: {:?}", proposal);
        self.governance_system.lock().map_err(|e| IcnError::Governance(e.to_string()))?.create_proposal(proposal)
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        info!("Getting balance for address: {} and currency type: {:?}", address, currency_type);
        self.currency_system
            .lock()
            .map_err(|e| IcnError::Currency(e.to_string()))?
            .get_balance(address, currency_type)
    }

    pub fn get_latest_block(&self) -> IcnResult<Block> {
        self.blockchain
            .lock()
            .map_err(|e| IcnError::Blockchain(e.to_string()))?
            .get_latest_block()
            .ok_or_else(|| IcnError::Blockchain("No blocks in the blockchain".into()))
    }

    pub fn execute_smart_contract(&self, contract_id: &str, input: &[u8]) -> IcnResult<Vec<u8>> {
        self.contract_manager
            .lock()
            .map_err(|e| IcnError::Vm(e.to_string()))?
            .execute_contract(contract_id, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icn_node_creation() {
        let config = Config {
            shard_count: 4,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
        };
        let node = IcnNode::new(config).unwrap();
        assert!(node.start().is_ok());
        assert!(node.stop().is_ok());
    }

    // Add more tests for other IcnNode functionalities
}