// File: icn_core/src/lib.rs

use icn_common::{Config, Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType, ProposalStatus, IcnResult, IcnError};
use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::GovernanceSystem;
use icn_identity::IdentityService;
use icn_network::NetworkManager;
use icn_sharding::ShardingManager;
use icn_vm::SmartContractExecutor;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub struct IcnNode {
    config: Config,
    blockchain: Arc<RwLock<Blockchain>>,
    consensus: Arc<RwLock<PoCConsensus>>,
    currency_system: Arc<RwLock<CurrencySystem>>,
    governance: Arc<RwLock<GovernanceSystem>>,
    identity_service: Arc<RwLock<IdentityService>>,
    network_manager: Arc<RwLock<NetworkManager>>,
    sharding_manager: Arc<RwLock<ShardingManager>>,
    smart_contract_executor: Arc<RwLock<SmartContractExecutor>>,
}

impl IcnNode {
    pub async fn new(config: Config) -> IcnResult<Self> {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let consensus = Arc::new(RwLock::new(PoCConsensus::new(config.consensus_threshold, config.consensus_quorum)?));
        let currency_system = Arc::new(RwLock::new(CurrencySystem::new()));
        let governance = Arc::new(RwLock::new(GovernanceSystem::new()));
        let identity_service = Arc::new(RwLock::new(IdentityService::new()));
        let network_manager = Arc::new(RwLock::new(NetworkManager::new(config.network_port)));
        let sharding_manager = Arc::new(RwLock::new(ShardingManager::new(config.shard_count)));
        let smart_contract_executor = Arc::new(RwLock::new(SmartContractExecutor::new()));

        Ok(Self {
            config,
            blockchain,
            consensus,
            currency_system,
            governance,
            identity_service,
            network_manager,
            sharding_manager,
            smart_contract_executor,
        })
    }

    pub async fn start(&self) -> IcnResult<()> {
        self.consensus.write().await.start()?;
        self.network_manager.write().await.start()?;
        Ok(())
    }

    pub async fn stop(&self) -> IcnResult<()> {
        self.consensus.write().await.stop()?;
        self.network_manager.write().await.stop()?;
        Ok(())
    }

    pub async fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        let shard_id = self.sharding_manager.read().await.get_shard_for_address(&transaction.from);
        let mut blockchain = self.blockchain.write().await;
        blockchain.add_transaction(transaction.clone())?;
        
        let mut currency_system = self.currency_system.write().await;
        currency_system.process_transaction(&transaction)?;

        self.sharding_manager.write().await.process_transaction(shard_id, &transaction)?;

        Ok(())
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        self.governance.write().await.create_proposal(proposal)
    }

    pub async fn vote_on_proposal(&self, proposal_id: &str, voter: String, in_favor: bool, weight: f64) -> IcnResult<()> {
        self.governance.write().await.vote_on_proposal(proposal_id, voter, in_favor, weight)
    }

    pub async fn finalize_proposal(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        self.governance.write().await.finalize_proposal(proposal_id)
    }

    pub async fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currency_system.read().await.get_balance(address, currency_type)
    }

    pub async fn mint_currency(&self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        self.currency_system.write().await.mint(address, currency_type, amount)
    }

    pub async fn create_identity(&self, attributes: HashMap<String, String>) -> IcnResult<String> {
        self.identity_service.write().await.create_identity(attributes)
    }

    pub async fn get_identity(&self, id: &str) -> IcnResult<HashMap<String, String>> {
        self.identity_service.read().await.get_identity(id)
    }

    pub async fn update_identity(&self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        self.identity_service.write().await.update_identity(id, attributes)
    }

    pub async fn execute_smart_contract(&self, contract_id: &str, function: &str, args: Vec<icn_vm::Value>) -> IcnResult<Option<icn_vm::Value>> {
        self.smart_contract_executor.write().await.execute_contract(contract_id, function, args)
    }

    pub async fn get_blockchain(&self) -> IcnResult<Vec<icn_blockchain::Block>> {
        Ok(self.blockchain.read().await.get_chain().clone())
    }

    pub async fn get_network_stats(&self) -> IcnResult<icn_network::NetworkStats> {
        self.network_manager.read().await.get_stats()
    }

    pub async fn allocate_resource(&self, resource_type: &str, amount: u64) -> IcnResult<()> {
        self.sharding_manager.write().await.allocate_resource(resource_type, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_icn_node() {
        let config = Config {
            shard_count: 1,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
        };

        let node = IcnNode::new(config).await.unwrap();

        // Test identity creation
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        let identity_id = node.create_identity(attributes).await.unwrap();

        // Test transaction processing
        let transaction = Transaction {
            from: identity_id.clone(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        node.mint_currency(&identity_id, &CurrencyType::BasicNeeds, 1000.0).await.unwrap();
        node.process_transaction(transaction).await.unwrap();

        // Test proposal creation and voting
        let proposal = Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: identity_id.clone(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };

        let proposal_id = node.create_proposal(proposal).await.unwrap();

        node.vote_on_proposal(&proposal_id, identity_id.clone(), true, 1.0).await.unwrap();

        // Test closing proposal
        let proposal_status = node.finalize_proposal(&proposal_id).await.unwrap();
        assert_eq!(proposal_status, ProposalStatus::Passed);

        // Test balance check
        let balance = node.get_balance(&identity_id, &CurrencyType::BasicNeeds).await.unwrap();
        assert_eq!(balance, 900.0); // 1000 minted - 100 transferred

        // Test get blockchain
        let blockchain = node.get_blockchain().await.unwrap();
        assert_eq!(blockchain.len(), 2); // Genesis block + 1 transaction block

        // Test get identity
        let identity = node.get_identity(&identity_id).await.unwrap();
        assert_eq!(identity.get("name"), Some(&"Alice".to_string()));

        // Test update identity
        let mut new_attributes = HashMap::new();
        new_attributes.insert("age".to_string(), "30".to_string());
        node.update_identity(&identity_id, new_attributes).await.unwrap();

        let updated_identity = node.get_identity(&identity_id).await.unwrap();
        assert_eq!(updated_identity.get("age"), Some(&"30".to_string()));

        // Test smart contract execution
        let contract_id = "test_contract";
        let function = "test_function";
        let args = vec![icn_vm::Value::Int(42)];
        let result = node.execute_smart_contract(contract_id, function, args).await.unwrap();
        assert_eq!(result, Some(icn_vm::Value::Int(42)));

        // Test network stats
        let network_stats = node.get_network_stats().await.unwrap();
        assert!(network_stats.connected_peers >= 0);

        // Test resource allocation
        node.allocate_resource("computing_power", 100).await.unwrap();

        assert!(node.stop().await.is_ok());
    }
}