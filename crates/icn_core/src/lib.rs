use icn_common::{Config, Transaction, CurrencyType, ProposalStatus as CommonProposalStatus, NetworkStats, IcnResult, IcnError};
use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::{GovernanceSystem, Proposal, ProposalStatus};
use icn_identity::{IdentityService, DecentralizedIdentity};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use ed25519_dalek::Signature;

pub struct IcnNode {
    config: Config,
    blockchain: Arc<RwLock<Blockchain>>,
    consensus: Arc<RwLock<PoCConsensus>>,
    currency_system: Arc<RwLock<CurrencySystem>>,
    governance: Arc<RwLock<GovernanceSystem>>,
    identity_service: Arc<RwLock<IdentityService>>,
}

impl IcnNode {
    pub fn new(config: Config) -> IcnResult<Self> {
        let consensus = Arc::new(RwLock::new(PoCConsensus::new(config.consensus_threshold, config.consensus_quorum)?));
        Ok(Self {
            config,
            blockchain: Arc::new(RwLock::new(Blockchain::new())),
            consensus,
            currency_system: Arc::new(RwLock::new(CurrencySystem::new())),
            governance: Arc::new(RwLock::new(GovernanceSystem::new())),
            identity_service: Arc::new(RwLock::new(IdentityService::new())),
        })
    }

    pub async fn start(&self) -> IcnResult<()> {
        // Initialize and start necessary components
        self.consensus.read().await.start()?;
        Ok(())
    }

    pub async fn stop(&self) -> IcnResult<()> {
        // Stop and clean up components
        self.consensus.read().await.stop()?;
        Ok(())
    }

    pub async fn create_identity(&self, attributes: HashMap<String, String>) -> IcnResult<DecentralizedIdentity> {
        self.identity_service.write().await.create_identity(attributes)
    }

    pub async fn get_identity(&self, id: &str) -> IcnResult<DecentralizedIdentity> {
        self.identity_service.read().await.get_identity(id).cloned().unwrap_or_else(|| Err(IcnError::CustomError("Identity not found".to_string())))
    }

    pub async fn update_identity_attributes(&self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        self.identity_service.write().await.update_attributes(id, attributes)
    }

    pub async fn update_identity_reputation(&self, id: &str, change: f64) -> IcnResult<()> {
        self.identity_service.write().await.update_reputation(id, change)
    }

    pub async fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        let mut blockchain = self.blockchain.write().await;
        blockchain.add_transaction(transaction.clone())?;
        
        let mut currency_system = self.currency_system.write().await;
        currency_system.process_transaction(&transaction)?;

        Ok(())
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        self.governance.write().await.create_proposal(proposal)
    }

    pub async fn vote_on_proposal(&self, proposal_id: &str, voter: String, in_favor: bool, weight: f64) -> IcnResult<()> {
        self.governance.write().await.vote_on_proposal(proposal_id, voter, in_favor, weight)
    }

    pub async fn close_proposal(&self, proposal_id: &str) -> IcnResult<CommonProposalStatus> {
        let status = self.governance.write().await.finalize_proposal(proposal_id)?;
        match status {
            ProposalStatus::Active => Ok(CommonProposalStatus::Active),
            ProposalStatus::Passed => Ok(CommonProposalStatus::Passed),
            ProposalStatus::Rejected => Ok(CommonProposalStatus::Rejected),
        }
    }

    pub async fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currency_system.read().await.get_balance(address, currency_type)
    }

    pub async fn mint_currency(&self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        self.currency_system.write().await.mint(address, currency_type, amount)
    }

    pub async fn get_network_stats(&self) -> IcnResult<NetworkStats> {
        let blockchain = self.blockchain.read().await;
        let governance = self.governance.read().await;

        Ok(NetworkStats {
            node_count: 1, // For simplicity, we're assuming a single node in this demo
            total_transactions: blockchain.get_transaction_count(),
            active_proposals: governance.list_active_proposals().len(),
        })
    }

    pub async fn validate_block(&self, block: &icn_blockchain::Block) -> IcnResult<bool> {
        self.consensus.read().await.validate_block(block)
    }

    pub async fn process_new_block(&self, block: icn_blockchain::Block) -> IcnResult<()> {
        self.consensus.write().await.process_new_block(block)
    }

    pub async fn get_blockchain(&self) -> IcnResult<Vec<icn_blockchain::Block>> {
        Ok(self.blockchain.read().await.get_chain().clone())
    }

    pub async fn list_active_proposals(&self) -> IcnResult<Vec<Proposal>> {
        Ok(self.governance.read().await.list_active_proposals().into_iter().cloned().collect())
    }

    pub async fn get_proposal(&self, proposal_id: &str) -> IcnResult<Proposal> {
        self.governance.read().await.get_proposal(proposal_id).cloned().unwrap_or_else(|| Err(IcnError::CustomError("Proposal not found".to_string())))
    }

    pub async fn verify_signature(&self, id: &str, message: &[u8], signature: &Signature) -> IcnResult<bool> {
        self.identity_service.read().await.verify_signature(id, message, signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{ProposalType, ProposalCategory};
    use chrono::Utc;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_icn_node() {
        let config = Config {
            shard_count: 1,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
        };

        let node = IcnNode::new(config).unwrap();

        // Test identity creation
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        let identity = node.create_identity(attributes).await.unwrap();

        // Test transaction processing
        let transaction = Transaction {
            from: identity.id.clone(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None, // In a real scenario, this should be signed
        };

        node.mint_currency(&identity.id, &CurrencyType::BasicNeeds, 1000.0).await.unwrap();
        node.process_transaction(transaction).await.unwrap();

        // Test proposal creation and voting
        let proposal = Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: identity.id.clone(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };

        let proposal_id = node.create_proposal(proposal).await.unwrap();

        let vote = icn_governance::Vote {
            voter: identity.id.clone(),
            proposal_id: proposal_id.clone(),
            in_favor: true,
            weight: 1.0,
            timestamp: Utc::now().timestamp(),
        };

        node.vote_on_proposal(&vote.proposal_id, vote.voter.clone(), vote.in_favor, vote.weight).await.unwrap();

        // Test closing proposal
        let proposal_status = node.close_proposal(&proposal_id).await.unwrap();
        assert_eq!(proposal_status, CommonProposalStatus::Passed);

        // Test balance check
        let balance = node.get_balance(&identity.id, &CurrencyType::BasicNeeds).await.unwrap();
        assert_eq!(balance, 900.0); // 1000 minted - 100 transferred

        // Test network stats
        let stats = node.get_network_stats().await.unwrap();
        assert_eq!(stats.node_count, 1);
        assert_eq!(stats.total_transactions, 1);
        assert_eq!(stats.active_proposals, 0); // The proposal was closed

        // Test blockchain
        let blockchain = node.get_blockchain().await.unwrap();
        assert_eq!(blockchain.len(), 1); // Genesis block

        // Test active proposals
        let active_proposals = node.list_active_proposals().await.unwrap();
        assert_eq!(active_proposals.len(), 0);

        // Test get proposal
        let retrieved_proposal = node.get_proposal(&proposal_id).await.unwrap();
        assert_eq!(retrieved_proposal.id, proposal_id);

        // Test signature verification
        let message = b"Test message";
        let keypair = ed25519_dalek::Keypair::generate(&mut rand::rngs::OsRng);
        let signature = keypair.sign(message);
        let verify_result = node.verify_signature(&identity.id, message, &signature).await.unwrap();
        assert!(!verify_result); // Should be false as we used a different keypair

        // Test update identity attributes
        let mut new_attributes = HashMap::new();
        new_attributes.insert("age".to_string(), "30".to_string());
        node.update_identity_attributes(&identity.id, new_attributes).await.unwrap();

        let updated_identity = node.get_identity(&identity.id).await.unwrap();
        assert_eq!(updated_identity.attributes.get("age"), Some(&"30".to_string()));

        // Test update identity reputation
        node.update_identity_reputation(&identity.id, 0.5).await.unwrap();
        let updated_identity = node.get_identity(&identity.id).await.unwrap();
        assert_eq!(updated_identity.reputation, 1.5);
    }
}
