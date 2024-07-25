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
use log::{info, warn, error};
use std::collections::HashMap;
use chrono::{Duration, Utc};
use uuid::Uuid;


pub struct Config {
    pub shard_count: u64,
    pub consensus_threshold: f64,
    pub consensus_quorum: f64,
    pub network_port: u16,
    pub zkp_bit_size: usize,
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
        let zkp_manager = Arc::new(RwLock::new(ZKPManager::new(config.zkp_bit_size)));

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
        info!("Starting InterCooperative Network node");
        self.blockchain.read().unwrap().start()?;
        self.consensus.read().unwrap().start()?;
        self.network.lock().await.start().await?;
        self.listen_for_network_events();
        Ok(())
    }

    pub async fn stop(&self) -> IcnResult<()> {
        info!("Stopping InterCooperative Network node");
        self.blockchain.read().unwrap().stop()?;
        self.consensus.read().unwrap().stop()?;
        self.network.lock().await.stop().await?;
        Ok(())
    }

    pub async fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        self.verify_transaction(&transaction)?;
        
        // Create ZKP proof
        let zkp_proof = self.zkp_manager.read().unwrap().create_proof(&transaction)?;
        
        // Verify ZKP proof
        if !self.zkp_manager.read().unwrap().verify_proof(&zkp_proof, &transaction)? {
            return Err(IcnError::ZKP("Invalid ZKP proof".into()));
        }

        let from_shard = self.sharding_manager.read().unwrap().get_shard_for_address(&transaction.from);
        let to_shard = self.sharding_manager.read().unwrap().get_shard_for_address(&transaction.to);

        if from_shard != to_shard {
            self.process_cross_shard_transaction(transaction, from_shard, to_shard).await?;
        } else {
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

    pub fn create_identity(&self, attributes: HashMap<String, String>) -> IcnResult<icn_identity::DecentralizedIdentity> {
        self.identity_manager.write().unwrap().create_identity(attributes)
    }

    pub fn allocate_resource(&self, resource_id: &str, amount: u64) -> IcnResult<()> {
        info!("Allocating {} units of resource {}", amount, resource_id);
        // Implement resource allocation logic here
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

    fn verify_transaction(&self, transaction: &Transaction) -> IcnResult<()> {
        // Implement transaction verification logic
        // This should include signature verification, balance checks, etc.
        Ok(())
    }

    async fn process_cross_shard_transaction(&self, transaction: Transaction, from_shard: u64, to_shard: u64) -> IcnResult<()> {
        // Implement cross-shard transaction processing logic
        // This should include locking funds in the source shard, transferring to the destination shard, and updating balances
        Ok(())
    }

    fn listen_for_network_events(&self) {
        // Implement network event listening logic
        // This should handle incoming messages, new peer connections, etc.
    }
}

pub struct NetworkStats {
    pub connected_peers: u32,
    pub total_transactions: u64,
    pub uptime: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_creation_and_basic_operations() {
        let config = Config {
            shard_count: 4,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
            zkp_bit_size: 64,
        };

        let node = IcnNode::new(config).unwrap();
        node.start().await.unwrap();

        // Test create identity
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        attributes.insert("email".to_string(), "alice@example.com".to_string());
        let identity = node.create_identity(attributes).unwrap();
        assert_eq!(identity.attributes.get("name"), Some(&"Alice".to_string()));

        // Test process transaction
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
        };
        assert!(node.process_transaction(transaction).await.is_ok());

// Test create proposal
let proposal = Proposal {
    id: Uuid::new_v4().to_string(),
    title: "Test Proposal".to_string(),
    description: "This is a test proposal".to_string(),
    proposer: "Alice".to_string(),
    created_at: Utc::now(),
    voting_ends_at: Utc::now() + Duration::days(7),
    status: icn_common::ProposalStatus::Active,
    proposal_type: icn_common::ProposalType::Constitutional,
    category: icn_common::ProposalCategory::Economic,
    required_quorum: 0.66,
    execution_timestamp: None,
};
assert!(node.create_proposal(proposal).is_ok());

// Test get network stats
let stats = node.get_network_stats().await.unwrap();
assert!(stats.connected_peers >= 0);

// Test allocate resource
assert!(node.allocate_resource("computing_power", 100).is_ok());

// Test get balance
let balance = node.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap();
assert!(balance >= 0.0);

node.stop().await.unwrap();
}

#[tokio::test]
async fn test_cross_shard_transaction() {
let config = Config {
    shard_count: 2,
    consensus_threshold: 0.66,
    consensus_quorum: 0.51,
    network_port: 8081,
    zkp_bit_size: 64,
};

let node = IcnNode::new(config).unwrap();
node.start().await.unwrap();

// Set up shards and initial balances
{
    let mut sharding_manager = node.sharding_manager.write().unwrap();
    sharding_manager.add_address_to_shard("Alice".to_string(), 0);
    sharding_manager.add_address_to_shard("Bob".to_string(), 1);
    sharding_manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0);
}

// Create and process a cross-shard transaction
let transaction = Transaction {
    from: "Alice".to_string(),
    to: "Bob".to_string(),
    amount: 500.0,
    currency_type: CurrencyType::BasicNeeds,
    timestamp: chrono::Utc::now().timestamp(),
    signature: None, // In a real scenario, this should be signed
};

assert!(node.process_transaction(transaction).await.is_ok());

// Verify balances after the transaction
let alice_balance = node.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap();
let bob_balance = node.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap();

assert_eq!(alice_balance, 500.0);
assert_eq!(bob_balance, 500.0);

node.stop().await.unwrap();
}

#[tokio::test]
async fn test_zkp_integration() {
let config = Config {
    shard_count: 1,
    consensus_threshold: 0.66,
    consensus_quorum: 0.51,
    network_port: 8082,
    zkp_bit_size: 64,
};

let node = IcnNode::new(config).unwrap();
node.start().await.unwrap();

// Set up initial balance
{
    let mut currency_system = node.currency_system.write().unwrap();
    currency_system.mint("Alice", CurrencyType::BasicNeeds, 1000.0).unwrap();
}

// Create and process a transaction with ZKP
let transaction = Transaction {
    from: "Alice".to_string(),
    to: "Bob".to_string(),
    amount: 100.0,
    currency_type: CurrencyType::BasicNeeds,
    timestamp: chrono::Utc::now().timestamp(),
    signature: None, // In a real scenario, this should be signed
};

assert!(node.process_transaction(transaction).await.is_ok());

// Verify balances after the transaction
let alice_balance = node.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap();
let bob_balance = node.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap();

assert_eq!(alice_balance, 900.0);
assert_eq!(bob_balance, 100.0);

node.stop().await.unwrap();
}