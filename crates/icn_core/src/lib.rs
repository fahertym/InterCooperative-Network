// File: crates/icn_core/src/lib.rs

use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::GovernanceSystem;
use icn_identity::IdentityManager;
use icn_network::Network;
use icn_sharding::ShardingManager;
use icn_storage::StorageManager;
use icn_vm::CoopVM;
use icn_zkp::ZKPManager;

use icn_common::{Block, Transaction, Proposal, IcnResult, IcnError, CurrencyType};
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex as AsyncMutex;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use log::{info, warn, error};
use uuid::Uuid;

pub struct Config {
    pub shard_count: u64,
    pub consensus_threshold: f64,
    pub consensus_quorum: f64,
    pub network_port: u16,
}

pub struct IcnNode {
    blockchain: Arc<RwLock<Blockchain>>,
    consensus: Arc<RwLock<PoCConsensus>>,
    currency_system: Arc<RwLock<CurrencySystem>>,
    governance: Arc<RwLock<GovernanceSystem>>,
    identity_manager: Arc<RwLock<IdentityManager>>,
    network: Arc<AsyncMutex<Network>>,
    sharding_manager: Arc<RwLock<ShardingManager>>,
    storage_manager: Arc<RwLock<StorageManager>>,
    vm: Arc<RwLock<CoopVM>>,
    zkp_manager: Arc<RwLock<ZKPManager>>,
}

impl IcnNode {
    pub fn new(config: Config) -> IcnResult<Self> {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()?));
        let consensus = Arc::new(RwLock::new(PoCConsensus::new(config.consensus_threshold, config.consensus_quorum)?));
        let currency_system = Arc::new(RwLock::new(CurrencySystem::new()));
        let governance = Arc::new(RwLock::new(GovernanceSystem::new(
            Arc::clone(&blockchain),
            Arc::clone(&consensus),
        )));
        let identity_manager = Arc::new(RwLock::new(IdentityManager::new()));
        let network = Arc::new(AsyncMutex::new(Network::new(format!("127.0.0.1:{}", config.network_port).parse().map_err(|e| IcnError::Network(e.to_string()))?)));
        let sharding_manager = Arc::new(RwLock::new(ShardingManager::new(config.shard_count)));
        let storage_manager = Arc::new(RwLock::new(StorageManager::new(3))); // Replication factor of 3
        let vm = Arc::new(RwLock::new(CoopVM::new(Vec::new()))); // Empty program for now
        let zkp_manager = Arc::new(RwLock::new(ZKPManager::new()?));

        Ok(IcnNode {
            blockchain,
            consensus,
            currency_system,
            governance,
            identity_manager,
            network,
            sharding_manager,
            storage_manager,
            vm,
            zkp_manager,
        })
    }

    pub async fn start(&self) -> IcnResult<()> {
        // Start all components
        self.blockchain.read().unwrap().start()?;
        self.consensus.read().unwrap().start()?;
        self.network.lock().await.start().await?;
        
        // Start listening for network events
        self.listen_for_network_events();

        Ok(())
    }

    pub async fn stop(&self) -> IcnResult<()> {
        // Stop all components
        self.blockchain.read().unwrap().stop()?;
        self.consensus.read().unwrap().stop()?;
        self.network.lock().await.stop().await?;

        Ok(())
    }

    pub async fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        // Verify the transaction
        self.verify_transaction(&transaction)?;

        // If it's a cross-shard transaction, handle it accordingly
        let from_shard = self.sharding_manager.read().unwrap().get_shard_for_address(&transaction.from);
        let to_shard = self.sharding_manager.read().unwrap().get_shard_for_address(&transaction.to);

        if from_shard != to_shard {
            self.process_cross_shard_transaction(transaction, from_shard, to_shard).await?;
        } else {
            // Add the transaction to the blockchain
            self.blockchain.write().unwrap().add_transaction(transaction)?;
        }

        Ok(())
    }

    pub fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        self.governance.write().unwrap().create_proposal(proposal)
    }

    pub fn vote_on_proposal(&self, proposal_id: &str, voter: &str, vote: bool) -> IcnResult<()> {
        self.governance.write().unwrap().vote_on_proposal(proposal_id, voter, vote)
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currency_system.read().unwrap().get_balance(address, currency_type)
    }

    pub fn create_identity(&self, attributes: HashMap<String, String>) -> IcnResult<DecentralizedIdentity> {
        self.identity_manager.write().unwrap().create_identity(attributes)
    }

    pub fn allocate_resource(&self, resource_id: &str, amount: u64) -> IcnResult<()> {
        // In a real implementation, this would interact with a resource management system
        // For now, we'll just log the allocation
        log::info!("Allocating {} units of resource {}", amount, resource_id);
        Ok(())
    }

    pub async fn get_network_stats(&self) -> IcnResult<NetworkStats> {
        let network = self.network.lock().await;
        Ok(NetworkStats {
            connected_peers: network.get_connected_peers().len() as u32,
            total_transactions: self.blockchain.read().unwrap().get_total_transactions(),
            uptime: network.get_uptime(),
        })
    }

    // Helper methods
    fn verify_transaction(&self, transaction: &Transaction) -> IcnResult<()> {
        // Check if the sender has sufficient balance
        let balance = self.get_balance(&transaction.from, &transaction.currency_type)?;
        if balance < transaction.amount {
            return Err(IcnError::InsufficientFunds);
        }

        // Verify the transaction signature
        if !transaction.verify()? {
            return Err(IcnError::InvalidSignature);
        }

        Ok(())
    }

    async fn process_cross_shard_transaction(&self, transaction: Transaction, from_shard: u64, to_shard: u64) -> IcnResult<()> {
        // Lock funds in the source shard
        self.sharding_manager.write().unwrap().lock_funds(from_shard, &transaction.from, transaction.amount)?;

        // Create a cross-shard transaction record
        let cross_shard_tx = self.sharding_manager.write().unwrap().create_cross_shard_transaction(transaction.clone(), from_shard, to_shard)?;

        // Broadcast the cross-shard transaction to the network
        self.network.lock().await.broadcast_cross_shard_transaction(cross_shard_tx).await?;

        Ok(())
    }

    fn listen_for_network_events(&self) {
        let blockchain = Arc::clone(&self.blockchain);
        let consensus = Arc::clone(&self.consensus);
        let network = Arc::clone(&self.network);

        tokio::spawn(async move {
            loop {
                let event = network.lock().await.receive_event().await;
                match event {
                    NetworkEvent::NewTransaction(transaction) => {
                        if let Err(e) = blockchain.write().unwrap().add_transaction(transaction) {
                            log::error!("Failed to add transaction: {:?}", e);
                        }
                    }
                    NetworkEvent::NewBlock(block) => {
                        if let Err(e) = consensus.write().unwrap().process_new_block(block) {
                            log::error!("Failed to process new block: {:?}", e);
                        }
                    }
                    NetworkEvent::ConsensusMessage(message) => {
                        if let Err(e) = consensus.write().unwrap().handle_consensus_message(message) {
                            log::error!("Failed to handle consensus message: {:?}", e);
                        }
                    }
                    NetworkEvent::PeerConnected(peer_id) => {
                        log::info!("New peer connected: {:?}", peer_id);
                    }
                    NetworkEvent::PeerDisconnected(peer_id) => {
                        log::info!("Peer disconnected: {:?}", peer_id);
                    }
                }
            }
        });
    }
}

pub struct NetworkStats {
    pub connected_peers: u32,
    pub total_transactions: u64,
    pub uptime: std::time::Duration,
}

pub struct DecentralizedIdentity {
    pub id: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug)]
pub enum NetworkEvent {
    NewTransaction(Transaction),
    NewBlock(Block),
    ConsensusMessage(ConsensusMessage),
    PeerConnected(String),
    PeerDisconnected(String),
}

#[derive(Debug)]
pub enum ConsensusMessage {
    // Define consensus message types here
    // For example: Proposal, Vote, Commit, etc.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_icn_node() {
        let config = Config {
            shard_count: 4,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
        };

        let node = IcnNode::new(config).unwrap();
        
        // Test starting and stopping the node
        assert!(node.start().await.is_ok());
        assert!(node.stop().await.is_ok());

        // Test creating a new identity
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        let identity = node.create_identity(attributes).unwrap();
        assert_eq!(identity.attributes.get("name"), Some(&"Alice".to_string()));

        // Test processing a transaction
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: chrono::Utc::now().timestamp(),
            signature: None, // In a real scenario, this should be properly signed
        };
        assert!(node.process_transaction(transaction).await.is_ok());

        // Test creating a proposal
        let proposal = Proposal {
            id: "proposal1".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: chrono::Utc::now(),
            voting_ends_at: chrono::Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };
        let proposal_id = node.create_proposal(proposal).unwrap();
        assert!(!proposal_id.is_empty());

        // Test voting on a proposal
        assert!(node.vote_on_proposal(&proposal_id, "Alice", true).is_ok());

        // Test getting network stats
        let stats = node.get_network_stats().await.unwrap();
        assert_eq!(stats.connected_peers, 0); // No peers in test environment
        assert_eq!(stats.total_transactions, 1);

        // Test allocating a resource
        assert!(node.allocate_resource("computing_power", 100).is_ok());

        // Test getting balance
        let balance = node.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap();
        assert!(balance >= 0.0);
    }
}