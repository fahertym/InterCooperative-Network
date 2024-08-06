// File: icn_core/src/lib.rs

use icn_common::{Config, Transaction, Proposal, ProposalStatus, Vote, CurrencyType, IcnResult, IcnError, NetworkStats};
use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::GovernanceSystem;
use icn_identity::IdentityService;
use icn_network::NetworkManager;
use icn_sharding::ShardingManager;
use icn_vm::SmartContractExecutor;
use icn_zkp::ZKPManager;
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
    zkp_manager: Arc<RwLock<ZKPManager>>,
    proposals: Arc<RwLock<HashMap<String, Proposal>>>, // Added for proposal management
}

impl IcnNode {
    pub async fn new(config: Config) -> IcnResult<Self> {
        let blockchain = Arc::new(RwLock::new(Blockchain::new(config.difficulty)));
        let consensus = Arc::new(RwLock::new(PoCConsensus::new(config.consensus_threshold, config.consensus_quorum)?));
        let currency_system = Arc::new(RwLock::new(CurrencySystem::new()));
        let governance = Arc::new(RwLock::new(GovernanceSystem::new()));
        let identity_service = Arc::new(RwLock::new(IdentityService::new()));
        let network_manager = Arc::new(RwLock::new(NetworkManager::new(config.network_port)));
        let sharding_manager = Arc::new(RwLock::new(ShardingManager::new(config.shard_count)));
        let smart_contract_executor = Arc::new(RwLock::new(SmartContractExecutor::new()));
        let zkp_manager = Arc::new(RwLock::new(ZKPManager::new(64))); // Assuming 64-bit range proofs
        let proposals = Arc::new(RwLock::new(HashMap::new())); // Initialize proposals

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
            zkp_manager,
            proposals, // Include proposals map
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
        // Verify the transaction
        self.verify_transaction(&transaction).await?;

        // Get the shard for the transaction
        let shard_id = self.sharding_manager.read().await.get_shard_for_address(&transaction.from);

        // Process the transaction in the blockchain
        self.blockchain.write().await.add_transaction(transaction.clone())?;
        
        // Update the currency system
        self.currency_system.write().await.process_transaction(&transaction)?;

        // Process the transaction in the shard
        self.sharding_manager.write().await.process_transaction(shard_id, &transaction)?;

        Ok(())
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        // Verify the proposal
        self.verify_proposal(&proposal).await?;

        // Create the proposal in the governance system
        let proposal_id = self.governance.write().await.create_proposal(proposal)?;

        // Broadcast the new proposal to the network
        self.network_manager.read().await.broadcast_proposal(&proposal_id)?;

        Ok(proposal_id)
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
        Ok(self.blockchain.read().await.chain.clone())
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

    // New function to get proposal status
    pub async fn get_proposal_status(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let proposal = self.governance.read().await.get_proposal(proposal_id)?
            .ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;
        Ok(proposal.status)
    }

    // Helper methods

    async fn verify_transaction(&self, transaction: &Transaction) -> IcnResult<()> {
        // Verify the transaction signature
        if !transaction.verify()? {
            return Err(IcnError::Blockchain("Invalid transaction signature".into()));
        }

        // Check if the sender has sufficient balance
        let sender_balance = self.get_balance(&transaction.from, &transaction.currency_type).await?;
        if sender_balance < transaction.amount {
            return Err(IcnError::Currency("Insufficient balance".into()));
        }

        Ok(())
    }

    async fn verify_proposal(&self, proposal: &Proposal) -> IcnResult<()> {
        // Check if the proposer exists
        if self.get_identity(&proposal.proposer).await.is_err() {
            return Err(IcnError::Governance("Proposer does not exist".into()));
        }

        // Additional checks can be added here, such as:
        // - Checking if the proposal type is valid
        // - Verifying the proposal's required quorum is within acceptable limits
        // - Ensuring the voting period is reasonable

        Ok(())
    }

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

    pub async fn get_node_reputation(&self, node_id: &str) -> IcnResult<f64> {
        self.consensus.read().await.get_node_reputation(node_id)
    }

    pub async fn update_node_reputation(&self, node_id: &str, change: f64) -> IcnResult<()> {
        self.consensus.write().await.update_node_reputation(node_id, change)
    }

    pub async fn get_shard_for_address(&self, address: &str) -> u64 {
        self.sharding_manager.read().await.get_shard_for_address(address)
    }

    pub async fn create_smart_contract(&self, code: String) -> IcnResult<String> {
        self.smart_contract_executor.write().await.create_contract(code)
    }

    pub async fn get_smart_contract(&self, contract_id: &str) -> IcnResult<Option<String>> {
        self.smart_contract_executor.read().await.get_contract(contract_id)
    }

    pub async fn update_smart_contract(&self, contract_id: &str, new_code: String) -> IcnResult<()> {
        self.smart_contract_executor.write().await.update_contract(contract_id, new_code)
    }

    pub async fn delete_smart_contract(&self, contract_id: &str) -> IcnResult<()> {
        self.smart_contract_executor.write().await.delete_contract(contract_id)
    }

    pub async fn create_zkp(&self, transaction: &Transaction) -> IcnResult<(Vec<u8>, Vec<u8>)> {
        let zkp_manager = self.zkp_manager.read().await;
        let (proof, committed_values) = zkp_manager.create_proof(transaction)?;
        Ok((proof.to_bytes(), serde_json::to_vec(&committed_values)?))
    }

    pub async fn verify_zkp(&self, proof: &[u8], committed_values: &[u8]) -> IcnResult<bool> {
        let zkp_manager = self.zkp_manager.read().await;
        let proof = bulletproofs::RangeProof::from_bytes(proof)?;
        let committed_values: Vec<curve25519_dalek::scalar::Scalar> = serde_json::from_slice(committed_values)?;
        zkp_manager.verify_proof(&proof, &committed_values)
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
            difficulty: 2,
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

        // Get proposal status
        let status = node.get_proposal_status(&proposal_id).await.unwrap();
        assert_eq!(status, ProposalStatus::Active);

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
    async fn test_resource_allocation() {
        let node = create_test_node().await;
        assert!(node.allocate_resource("computing_power", 100).await.is_ok());
        // In a real implementation, we would check if the resource was actually allocated
        // For now, we just check if the method call succeeds
    }

    #[tokio::test]
    async fn test_smart_contract_execution() {
        let node = create_test_node().await;

        // Create a simple smart contract
        let contract_code = r#"
            fn add(a: i64, b: i64) -> i64 {
                a + b
            }
        "#.to_string();
        let contract_id = node.create_smart_contract(contract_code).await.unwrap();

        // Execute the smart contract
        let result = node.execute_smart_contract(&contract_id, "add", vec![icn_vm::Value::Int(5), icn_vm::Value::Int(3)]).await.unwrap();
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
        assert!(stats.node_count >= 0);
        assert!(stats.total_transactions >= 0);
        assert!(stats.active_proposals >= 0);
    }

    #[tokio::test]
    async fn test_zkp_operations() {
        let node = create_test_node().await;

        // Create a transaction
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        // Create a ZKP for the transaction
        let (proof, committed_values) = node.create_zkp(&transaction).await.unwrap();

        // Verify the ZKP
        let is_valid = node.verify_zkp(&proof, &committed_values).await.unwrap();
        assert!(is_valid);

        // Test with tampered data
        let mut tampered_values = committed_values.clone();
        tampered_values[0] ^= 1; // Flip a bit
        let is_invalid = node.verify_zkp(&proof, &tampered_values).await.unwrap();
        assert!(!is_invalid);
    }

    #[tokio::test]
    async fn test_reputation_management() {
        let node = create_test_node().await;
        let node_id = "test_node";

        // Initial reputation should be 0 or a default value
        let initial_reputation = node.get_node_reputation(node_id).await.unwrap();
        assert_eq!(initial_reputation, 0.0);

        // Update reputation
        node.update_node_reputation(node_id, 0.5).await.unwrap();
        let updated_reputation = node.get_node_reputation(node_id).await.unwrap();
        assert_eq!(updated_reputation, 0.5);

        // Update reputation again
        node.update_node_reputation(node_id, -0.2).await.unwrap();
        let final_reputation = node.get_node_reputation(node_id).await.unwrap();
        assert_eq!(final_reputation, 0.3);
    }

    #[tokio::test]
    async fn test_sharding() {
        let node = create_test_node().await;
        let address = "test_address";

        let shard_id = node.get_shard_for_address(address).await;
        assert!(shard_id < node.get_shard_count().await);

        // Test balance across shards
        node.mint_currency(address, &CurrencyType::BasicNeeds, 100.0).await.unwrap();
        let total_balance = node.get_total_balance(address, &CurrencyType::BasicNeeds).await.unwrap();
        assert_eq!(total_balance, 100.0);
    }

    #[tokio::test]
    async fn test_smart_contract_lifecycle() {
        let node = create_test_node().await;

        // Create a smart contract
        let contract_code = r#"
            fn multiply(a: i64, b: i64) -> i64 {
                a * b
            }
        "#.to_string();
        let contract_id = node.create_smart_contract(contract_code.clone()).await.unwrap();

        // Get the smart contract
        let retrieved_code = node.get_smart_contract(&contract_id).await.unwrap();
        assert_eq!(retrieved_code, Some(contract_code));

        // Update the smart contract
        let new_contract_code = r#"
            fn multiply(a: i64, b: i64) -> i64 {
                a * b + 1
            }
        "#.to_string();
        node.update_smart_contract(&contract_id, new_contract_code.clone()).await.unwrap();

        // Verify the update
        let updated_code = node.get_smart_contract(&contract_id).await.unwrap();
        assert_eq!(updated_code, Some(new_contract_code));

        // Execute the updated contract
        let result = node.execute_smart_contract(&contract_id, "multiply", vec![icn_vm::Value::Int(5), icn_vm::Value::Int(3)]).await.unwrap();
        assert_eq!(result, Some(icn_vm::Value::Int(16))); // 5 * 3 + 1 = 16

        // Delete the smart contract
        node.delete_smart_contract(&contract_id).await.unwrap();

        // Verify deletion
        let deleted_code = node.get_smart_contract(&contract_id).await.unwrap();
        assert_eq!(deleted_code, None);
    }

    #[tokio::test]
    async fn test_proposal_voting_and_finalization() {
        let node = create_test_node().await;

        // Create a proposal
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

        let proposal_id = node.create_proposal(proposal).await.unwrap();

        // Vote on the proposal
        node.vote_on_proposal(&proposal_id, "Alice".to_string(), true, 0.3).await.unwrap();
        node.vote_on_proposal(&proposal_id, "Bob".to_string(), true, 0.3).await.unwrap();
        node.vote_on_proposal(&proposal_id, "Charlie".to_string(), false, 0.2).await.unwrap();

        // Check proposal status
        let proposals_with_status = node.list_active_proposals_with_status().await.unwrap();
        assert_eq!(proposals_with_status.len(), 1);
        let (retrieved_proposal, voting_status) = &proposals_with_status[0];
        assert_eq!(retrieved_proposal.id, proposal_id);
        assert!(*voting_status > 0.5); // 0.6 in favor out of 0.8 total votes

        // Finalize the proposal
        let final_status = node.finalize_proposal(&proposal_id).await.unwrap();
        assert_eq!(final_status, ProposalStatus::Passed);

        // Verify the proposal is no longer active
        let active_proposals = node.list_active_proposals().await.unwrap();
        assert_eq!(active_proposals.len(), 0);
    }

    #[tokio::test]
    async fn test_cross_shard_balance() {
        let mut config = Config {
            shard_count: 2,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
            difficulty: 2,
        };
        let node = IcnNode::new(config).await.unwrap();

        let address = "cross_shard_user";

        // Mint currency in both shards
        node.mint_currency(address, &CurrencyType::BasicNeeds, 50.0).await.unwrap();
        node.mint_currency(address, &CurrencyType::BasicNeeds, 50.0).await.unwrap();

        // Check total balance across shards
        let total_balance = node.get_total_balance(address, &CurrencyType::BasicNeeds).await.unwrap();
        assert_eq!(total_balance, 100.0);

        // Perform a cross-shard transaction
        let transaction = Transaction {
            from: address.to_string(),
            to: "another_user".to_string(),
            amount: 75.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        assert!(node.process_transaction(transaction).await.is_ok());

        // Verify the balance after cross-shard transaction
        let new_balance = node.get_total_balance(address, &CurrencyType::BasicNeeds).await.unwrap();
        assert_eq!(new_balance, 25.0);
    }
}
