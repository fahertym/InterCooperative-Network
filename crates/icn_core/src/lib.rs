use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::GovernanceSystem;
use icn_identity::IdentityManager;
use icn_common::{IcnResult, IcnError, Transaction, Proposal, Block, CurrencyType};
use std::sync::{Arc, RwLock};

pub struct IcnNode {
    blockchain: Arc<RwLock<Blockchain>>,
    consensus: Arc<RwLock<PoCConsensus>>,
    currency_system: Arc<RwLock<CurrencySystem>>,
    governance: Arc<RwLock<GovernanceSystem>>,
    identity_manager: Arc<RwLock<IdentityManager>>,
}

impl IcnNode {
    pub fn new() -> IcnResult<Self> {
        Ok(IcnNode {
            blockchain: Arc::new(RwLock::new(Blockchain::new()?)),
            consensus: Arc::new(RwLock::new(PoCConsensus::new(0.66, 0.51)?)),
            currency_system: Arc::new(RwLock::new(CurrencySystem::new())),
            governance: Arc::new(RwLock::new(GovernanceSystem::new())),
            identity_manager: Arc::new(RwLock::new(IdentityManager::new())),
        })
    }

    pub fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        // Verify transaction
        self.verify_transaction(&transaction)?;

        // Process transaction in currency system
        self.currency_system.write().unwrap().process_transaction(&transaction)?;

        // Add transaction to blockchain
        self.blockchain.write().unwrap().add_transaction(transaction)?;

        Ok(())
    }

    pub fn create_identity(&self, attributes: std::collections::HashMap<String, String>) -> IcnResult<icn_identity::DecentralizedIdentity> {
        self.identity_manager.write().unwrap().create_identity(attributes)
    }

    pub fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        self.governance.write().unwrap().create_proposal(proposal)
    }

    pub fn vote_on_proposal(&self, proposal_id: &str, voter: String, in_favor: bool) -> IcnResult<()> {
        self.governance.write().unwrap().vote_on_proposal(proposal_id, voter, in_favor)
    }

    pub fn finalize_proposal(&self, proposal_id: &str) -> IcnResult<icn_common::ProposalStatus> {
        self.governance.write().unwrap().finalize_proposal(proposal_id)
    }

    pub fn get_balance(&self, account_id: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currency_system.read().unwrap().get_balance(account_id, currency_type)
    }

    pub fn create_block(&self) -> IcnResult<Block> {
        let block = self.blockchain.write().unwrap().create_block()?;
        self.consensus.read().unwrap().validate_block(&block)?;
        Ok(block)
    }

    fn verify_transaction(&self, transaction: &Transaction) -> IcnResult<()> {
        // In a real implementation, you would verify the transaction signature here
        // For now, we'll just check if the sender has sufficient balance
        let balance = self.get_balance(&transaction.from, &transaction.currency_type)?;
        if balance < transaction.amount {
            return Err(IcnError::Currency("Insufficient balance".into()));
        }
        Ok(())
    }
}