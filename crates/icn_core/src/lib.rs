//! Core functionality for the InterCooperative Network
//! 
//! This module provides the main `IcnNode` struct and associated functionality
//! for managing the core operations of the InterCooperative Network.

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

/// The main node structure for the InterCooperative Network
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
    /// Create a new IcnNode with the given configuration
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

    /// Start the IcnNode and its components
    pub fn start(&self) -> IcnResult<()> {
        info!("Starting ICN Node components");
        self.network.lock().map_err(|e| IcnError::LockError(e.to_string()))?.start()?;
        self.blockchain.lock().map_err(|e| IcnError::LockError(e.to_string()))?.start()?;
        self.consensus.lock().map_err(|e| IcnError::LockError(e.to_string()))?.start()?;
        // Add start methods for other components as needed
        info!("ICN Node started successfully");
        Ok(())
    }

    /// Stop the IcnNode and its components
    pub fn stop(&self) -> IcnResult<()> {
        info!("Stopping ICN Node components");
        self.network.lock().map_err(|e| IcnError::LockError(e.to_string()))?.stop()?;
        self.blockchain.lock().map_err(|e| IcnError::LockError(e.to_string()))?.stop()?;
        self.consensus.lock().map_err(|e| IcnError::LockError(e.to_string()))?.stop()?;
        // Add stop methods for other components as needed
        info!("ICN Node stopped successfully");
        Ok(())
    }

    /// Process a transaction
    pub fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        info!("Processing transaction: {:?}", transaction);
        let mut blockchain = self.blockchain.lock().map_err(|e| IcnError::LockError(e.to_string()))?;
        let mut currency_system = self.currency_system.lock().map_err(|e| IcnError::LockError(e.to_string()))?;

        if !self.verify_transaction(&transaction)? {
            return Err(IcnError::InvalidTransaction("Transaction verification failed".into()));
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

    /// Verify a transaction
    fn verify_transaction(&self, transaction: &Transaction) -> IcnResult<bool> {
        let sender_balance = self.get_balance(&transaction.from, &transaction.currency_type)?;
        if sender_balance < transaction.amount {
            return Ok(false);
        }

        // TODO: Implement signature verification

        Ok(true)
    }

    /// Create a new proposal
    pub fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        info!("Creating proposal: {:?}", proposal);
        self.governance_system.lock().map_err(|e| IcnError::LockError(e.to_string()))?.create_proposal(
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

    /// Get the balance for a given address and currency type
    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        info!("Getting balance for address: {} and currency type: {:?}", address, currency_type);
        self.currency_system
            .lock()
            .map_err(|e| IcnError::LockError(e.to_string()))?
            .get_balance(address, currency_type)
    }
}
