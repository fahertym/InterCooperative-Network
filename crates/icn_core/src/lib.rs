// File: crates/icn_core/src/lib.rs

use icn_common::{Config, Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType, ProposalStatus, IcnResult, IcnError};
use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::GovernanceSystem;
use icn_identity::IdentityService;
use icn_network::NetworkManager;
use icn_sharding::ShardingManager;
use icn_vm::SmartContractExecutor;
use icn_reputation::ReputationSystem;
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
    reputation_manager: Arc<RwLock<ReputationSystem>>,
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
        let reputation_manager = Arc::new(RwLock::new(ReputationSystem::new(0.01, 0.0, 100.0)));

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
            reputation_manager,
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

    pub async fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currency_system.read().await.get_balance(address, currency_type)
    }

    pub async fn create_identity(&self, attributes: HashMap<String, String>) -> IcnResult<String> {
        self.identity_service.write().await.create_identity(attributes)
    }

    pub async fn allocate_resource(&self, resource_type: &str, amount: u64) -> IcnResult<()> {
        self.sharding_manager.write().await.allocate_resource(resource_type, amount)
    }

    pub async fn get_network_stats(&self) -> IcnResult<NetworkStats> {
        self.network_manager.read().await.get_stats()
    }

    pub async fn get_proposal(&self, proposal_id: &str) -> IcnResult<Option<Proposal>> {
        self.governance.read().await.get_proposal(proposal_id)
    }

    pub async fn list_active_proposals(&self) -> IcnResult<Vec<Proposal>> {
        self.governance.read().await.list_active_proposals()
    }

    pub async fn vote_on_proposal(&self, proposal_id: &str, voter: String, in_favor: bool, weight: f64) -> IcnResult<()> {
        self.governance.write().await.vote_on_proposal(proposal_id, voter, in_favor, weight)
    }

    pub async fn finalize_proposal(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        self.governance.write().await.finalize_proposal(proposal_id)
    }

    pub async fn mint_currency(&self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        self.currency_system.write().await.mint(address, currency_type, amount)
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

    pub async fn get_shard_count(&self) -> u64 {
        self.config.shard_count
    }

    pub async fn get_consensus_threshold(&self) -> f64 {
        self.config.consensus_threshold
    }

    pub async fn get_consensus_quorum(&self) -> f64 {
        self.config.consensus_quorum
    }

    pub async fn get_network_port(&self) -> u16 {
        self.config.network_port
    }

    // Helper methods

    pub async fn get_total_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let mut total_balance = 0.0;
        for shard_id in 0..self.config.shard_count {
            total_balance += self.sharding_manager.read().await.get_shard_balance(shard_id, address, currency_type)?;
        }
        Ok(total_balance)
    }

    pub async fn list_active_proposals_with_status(&self) -> IcnResult<Vec<(Proposal, f64)>> {
        let proposals = self.governance.read().await.list_active_proposals()?;
        let mut proposals_with_status = Vec::new();
        
        for proposal in proposals {
            let votes = self.governance.read().await.get_votes(&proposal.id)?;
            let total_votes: f64 = votes.iter().map(|v| v.weight).sum();
            let votes_in_favor: f64 = votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();
            let status = if total_votes > 0.0 { votes_in_favor / total_votes } else { 0.0 };
            proposals_with_status.push((proposal, status));
        }
        
        Ok(proposals_with_status)
    }

    pub async fn check_sufficient_balance(&self, address: &str, amount: f64, currency_type: &CurrencyType) -> IcnResult<bool> {
        let balance = self.get_total_balance(address, currency_type).await?;
        Ok(balance >= amount)
    }

    pub async fn get_reputation_score(&self, address: &str) -> IcnResult<f64> {
        self.reputation_manager.read().await.get_score(address)
    }

    pub async fn update_reputation_score(&self, address: &str, change: f64) -> IcnResult<()> {
        self.reputation_manager.write().await.update_score(address, change)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    async fn create_test_node() -> IcnNode {
        let config = Config {
            shard_count: 1,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
        };
        IcnNode::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_node_creation_and_lifecycle() {
        let node = create_test_node().await;
        assert_eq!(node.get_shard_count().await, 1);
        assert_eq!(node.get_consensus_threshold().await, 0.66);
        assert_eq!(node.get_consensus_quorum().await, 0.51);
        assert_eq!(node.get_network_port().await, 8080);

        assert!(node.start().await.is_ok());
        assert!(node.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_processing() {
        let node = create_test_node().await;
        
        // Mint some currency for testing
        assert!(node.mint_currency("Alice", &CurrencyType::BasicNeeds, 1000.0).await.is_ok());

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        assert!(node.process_transaction(transaction).await.is_ok());

        // Check balances
        let alice_balance = node.get_balance("Alice", &CurrencyType::BasicNeeds).await.unwrap();
        let bob_balance = node.get_balance("Bob", &CurrencyType::BasicNeeds).await.unwrap();
        assert_eq!(alice_balance, 900.0);
        assert_eq!(bob_balance, 100.0);
    }

    #[tokio::test]
    async fn test_proposal_lifecycle() {
        let node = create_test_node().await;

        let proposal = Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.51,
            execution_timestamp: None,
        };

        // Create proposal
        let proposal_id = node.create_proposal(proposal).await.unwrap();

        // Check if proposal exists
        let retrieved_proposal = node.get_proposal(&proposal_id).await.unwrap();
        assert!(retrieved_proposal.is_some());

        // List active proposals
        let active_proposals = node.list_active_proposals().await.unwrap();
        assert_eq!(active_proposals.len(), 1);

        // Vote on proposal
        assert!(node.vote_on_proposal(&proposal_id, "Alice".to_string(), true, 1.0).await.is_ok());
        assert!(node.vote_on_proposal(&proposal_id, "Bob".to_string(), false, 1.0).await.is_ok());

        // Finalize proposal
        let final_status = node.finalize_proposal(&proposal_id).await.unwrap();
        assert_eq!(final_status, ProposalStatus::Passed);
    }

    #[tokio::test]
    async fn test_identity_management() {
        let node = create_test_node().await;

        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        attributes.insert("age".to_string(), "30".to_string());

        // Create identity
        let identity_id = node.create_identity(attributes.clone()).await.unwrap();

        // Get identity
        let retrieved_attributes = node.get_identity(&identity_id).await.unwrap();
        assert_eq!(retrieved_attributes, attributes);

        // Update identity
        attributes.insert("location".to_string(), "New York".to_string());
        assert!(node.update_identity(&identity_id, attributes.clone()).await.is_ok());

        // Verify updated identity
        let updated_attributes = node.get_identity(&identity_id).await.unwrap();
        assert_eq!(updated_attributes, attributes);
    }

    #[tokio::test]
    async fn test_helper_functions() {
        let node = create_test_node().await;
        
        // Test get_total_balance
        node.mint_currency("Alice", &CurrencyType::BasicNeeds, 100.0).await.unwrap();
        let balance = node.get_total_balance("Alice", &CurrencyType::BasicNeeds).await.unwrap();
        assert_eq!(balance, 100.0);

        // Test list_active_proposals_with_status
        let proposal = Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.51,
            execution_timestamp: None,
        };
        node.create_proposal(proposal).await.unwrap();
        let proposals_with_status = node.list_active_proposals_with_status().await.unwrap();
        assert_eq!(proposals_with_status.len(), 1);
        assert!(proposals_with_status[0].1 == 0.0); // No votes yet, so status should be 0.0

        // Test check_sufficient_balance
        assert!(node.check_sufficient_balance("Alice", 50.0, &CurrencyType::BasicNeeds).await.unwrap());
        assert!(!node.check_sufficient_balance("Alice", 150.0, &CurrencyType::BasicNeeds).await.unwrap());

        // Test reputation functions
        node.update_reputation_score("Alice", 5.0).await.unwrap();
        let score = node.get_reputation_score("Alice").await.unwrap();
        assert_eq!(score, 5.0);
    }

    #[tokio::test]
    async fn test_smart_contract_execution() {
        let node = create_test_node().await;

        // For this test, we'll assume a simple smart contract that adds two numbers
        let contract_id = "test_contract";
        let function = "add";
        let args = vec![icn_vm::Value::Int(5), icn_vm::Value::Int(3)];

        let result = node.execute_smart_contract(contract_id, function, args).await.unwrap();
        assert_eq!(result, Some(icn_vm::Value::Int(8)));
    }

    #[tokio::test]
    async fn test_blockchain_operations() {
        let node = create_test_node().await;

        // Add some transactions to create blocks
        for i in 0..5 {
            let transaction = Transaction {
                from: format!("User{}", i),
                to: format!("User{}", i+1),
                amount: 10.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: Utc::now().timestamp(),
                signature: None,
            };
            node.process_transaction(transaction).await.unwrap();
        }

        let blockchain = node.get_blockchain().await.unwrap();
        assert!(blockchain.len() > 1); // At least genesis block and one more
    }

    #[tokio::test]
    async fn test_network_stats() {
        let node = create_test_node().await;
        let stats = node.get_network_stats().await.unwrap();
        assert!(stats.connected_peers >= 0);
        assert!(stats.total_transactions >= 0);
        assert!(stats.active_proposals >= 0);
    }

    #[tokio::test]
    async fn test_resource_allocation() {
        let node = create_test_node().await;
        assert!(node.allocate_resource("computing_power", 100).await.is_ok());
        // In a real implementation, we would check if the resource was actually allocated
        // For now, we just check if the method call succeeds
    }
}