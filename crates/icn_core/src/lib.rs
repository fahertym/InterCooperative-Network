// File: /home/matt/InterCooperative-Network/crates/icn_core/src/lib.rs

use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::GovernanceSystem;
use icn_identity::IdentityManager;
use icn_common::{IcnResult, IcnError, Transaction, Proposal, Block, CurrencyType, ProposalStatus};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct IcnNode {
    blockchain: Arc<RwLock<Blockchain>>,
    consensus: Arc<RwLock<PoCConsensus>>,
    currency_system: Arc<RwLock<CurrencySystem>>,
    governance: Arc<RwLock<GovernanceSystem>>,
    identity_manager: Arc<RwLock<IdentityManager>>,
}

impl IcnNode {
    pub async fn new() -> IcnResult<Self> {
        Ok(IcnNode {
            blockchain: Arc::new(RwLock::new(Blockchain::new())),
            consensus: Arc::new(RwLock::new(PoCConsensus::new(0.66, 0.51)?)),
            currency_system: Arc::new(RwLock::new(CurrencySystem::new())),
            governance: Arc::new(RwLock::new(GovernanceSystem::new())),
            identity_manager: Arc::new(RwLock::new(IdentityManager::new())),
        })
    }

    pub async fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        // Verify transaction
        self.verify_transaction(&transaction).await?;

        // Process transaction in currency system
        self.currency_system.write().await.process_transaction(&transaction)?;

        // Add transaction to blockchain
        self.blockchain.write().await.add_transaction(transaction)?;

        Ok(())
    }

    pub async fn create_identity(&self, attributes: std::collections::HashMap<String, String>) -> IcnResult<icn_identity::DecentralizedIdentity> {
        self.identity_manager.write().await.create_identity(attributes)
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        self.governance.write().await.create_proposal(proposal)
    }

    pub async fn vote_on_proposal(&self, proposal_id: &str, voter: String, in_favor: bool) -> IcnResult<()> {
        self.governance.write().await.vote_on_proposal(proposal_id, voter, in_favor)
    }

    pub async fn finalize_proposal(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        self.governance.write().await.finalize_proposal(proposal_id)
    }

    pub async fn get_balance(&self, account_id: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currency_system.read().await.get_balance(account_id, currency_type)
    }

    pub async fn create_block(&self) -> IcnResult<Block> {
        let mut blockchain = self.blockchain.write().await;
        let block = blockchain.create_block()?;
        
        // Validate the block using the consensus mechanism
        self.consensus.read().await.validate_block(&block)?;
        
        Ok(block)
    }

    async fn verify_transaction(&self, transaction: &Transaction) -> IcnResult<()> {
        // Check if the sender has sufficient balance
        let balance = self.get_balance(&transaction.from, &transaction.currency_type).await?;
        if balance < transaction.amount {
            return Err(IcnError::Currency("Insufficient balance".into()));
        }

        // TODO: Implement signature verification
        // For now, we'll assume all transactions without signatures are valid
        if transaction.signature.is_none() {
            return Ok(());
        }

        // In a real implementation, you would verify the transaction signature here
        // For example:
        // self.identity_manager.read().await.verify_signature(&transaction.from, &transaction.hash(), &transaction.signature.unwrap())?;

        Ok(())
    }

    pub async fn get_latest_block(&self) -> IcnResult<Block> {
        self.blockchain.read().await.get_latest_block()
            .cloned()
            .ok_or_else(|| IcnError::Blockchain("No blocks in the chain".into()))
    }

    pub async fn get_blockchain_height(&self) -> u64 {
        self.blockchain.read().await.chain.len() as u64
    }

    pub async fn validate_blockchain(&self) -> bool {
        self.blockchain.read().await.validate_chain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::CurrencyType;

    #[tokio::test]
    async fn test_icn_node() {
        let node = IcnNode::new().await.unwrap();

        // Test create identity
        let alice_identity = node.create_identity(HashMap::new()).await.unwrap();
        let bob_identity = node.create_identity(HashMap::new()).await.unwrap();

        // Test process transaction
        let transaction = Transaction {
            from: alice_identity.id.clone(),
            to: bob_identity.id.clone(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
        };

        // First, mint some currency for Alice
        node.currency_system.write().await.mint(&alice_identity.id, &CurrencyType::BasicNeeds, 100.0).unwrap();

        // Now process the transaction
        node.process_transaction(transaction).await.unwrap();

        // Check balances
        let alice_balance = node.get_balance(&alice_identity.id, &CurrencyType::BasicNeeds).await.unwrap();
        let bob_balance = node.get_balance(&bob_identity.id, &CurrencyType::BasicNeeds).await.unwrap();

        assert_eq!(alice_balance, 50.0);
        assert_eq!(bob_balance, 50.0);

        // Test create block
        let block = node.create_block().await.unwrap();
        assert_eq!(block.index, 1);
        assert_eq!(block.transactions.len(), 1);

        // Test blockchain validation
        assert!(node.validate_blockchain().await);
    }
}