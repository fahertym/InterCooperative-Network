mod error;
pub use error::{Error, Result};

use icn_utils::{Blockchain, Block, Transaction, PoCConsensus, CurrencySystem, DemocraticSystem, DecentralizedIdentity, Network, ShardingManager, CoopVM};

pub struct IcnNode {
    pub blockchain: Blockchain,
    pub consensus: PoCConsensus,
    pub currency_system: CurrencySystem,
    pub democratic_system: DemocraticSystem,
    pub identity: DecentralizedIdentity,
    pub network: Network,
    pub sharding_manager: ShardingManager,
    pub vm: CoopVM,
}

impl IcnNode {
    pub fn new() -> Self {
        IcnNode {
            blockchain: Blockchain::new(),
            consensus: PoCConsensus::new(),
            currency_system: CurrencySystem::new(),
            democratic_system: DemocraticSystem::new(),
            identity: DecentralizedIdentity::new(),
            network: Network::new(),
            sharding_manager: ShardingManager::new(),
            vm: CoopVM::new(),
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<()> {
        self.blockchain.add_transaction(transaction)
    }

    pub fn create_block(&mut self) -> Result<()> {
        let pending_transactions = self.blockchain.pending_transactions().clone();
        let previous_hash = self.blockchain.latest_block().hash.clone();
        let new_block = Block::new(
            self.blockchain.chain().len() as u64,
            pending_transactions,
            previous_hash,
        );

        // Collect votes from validators
        let validators = self.consensus.get_validators();
        let votes: Vec<(&str, bool)> = validators.iter()
            .map(|v| (v.id.as_str(), self.collect_vote(&new_block, v)))
            .collect();

        // Add the block to the blockchain
        self.blockchain.add_block(new_block, &votes)
    }

    fn collect_vote(&self, block: &Block, validator: &icn_utils::Member) -> bool {
        // In a real implementation, this would involve network communication
        // and potentially running the block through the VM to verify its validity
        // For now, we'll just return true as a placeholder
        true
    }

    pub fn execute_smart_contract(&mut self, contract: String) -> Result<()> {
        let compiled_contract = self.vm.compile(contract)?;
        self.vm.execute(compiled_contract)
    }

    pub fn create_proposal(&mut self, proposal: String) -> Result<String> {
        self.democratic_system.create_proposal(proposal)
    }

    pub fn vote_on_proposal(&mut self, proposal_id: &str, vote: bool) -> Result<()> {
        self.democratic_system.vote(proposal_id, vote)
    }

    pub fn allocate_resources(&mut self, resource_type: String, amount: u64) -> Result<()> {
        self.sharding_manager.allocate_resources(resource_type, amount)
    }

    pub fn update_reputation(&mut self, address: &str, change: f64) -> Result<()> {
        self.consensus.update_reputation(address, change)
    }

    pub fn get_balance(&self, address: &str) -> Result<f64> {
        self.blockchain.get_balance(address)
    }

    pub fn create_asset_token(&mut self, name: String, description: String, owner: String) -> Result<String> {
        let token = self.blockchain.create_asset_token(name, description, owner)?;
        Ok(token.id)
    }

    pub fn transfer_asset_token(&mut self, token_id: &str, new_owner: &str) -> Result<()> {
        self.blockchain.transfer_asset_token(token_id, new_owner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_utils::CurrencyType;

    #[test]
    fn test_icn_node_creation() {
        let node = IcnNode::new();
        assert_eq!(node.blockchain.chain().len(), 1);
    }

    #[test]
    fn test_process_transaction() {
        let mut node = IcnNode::new();
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
        );
        assert!(node.process_transaction(transaction).is_ok());
    }

    #[test]
    fn test_create_block() {
        let mut node = IcnNode::new();
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
        );
        node.process_transaction(transaction).unwrap();
        assert!(node.create_block().is_ok());
        assert_eq!(node.blockchain.chain().len(), 2);
    }
}
