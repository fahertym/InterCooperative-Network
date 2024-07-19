// crates/icn_core/src/lib.rs

use std::sync::{Arc, RwLock};

// Re-export key types from other crates
pub use icn_blockchain::{Block, Transaction, Blockchain};
pub use icn_currency::CurrencyType;
pub use icn_governance::{DemocraticSystem, ProposalCategory, ProposalType};
pub use icn_identity::DecentralizedIdentity;
pub use icn_network::{Node as NetworkNode, Network, Packet, PacketType};
pub use icn_node::{ContentStore, ForwardingInformationBase, PendingInterestTable};
pub use icn_vm::{CoopVM, Opcode};
pub use icn_sharding::ShardingManager;

mod error;
pub use error::{Error, Result};

pub struct IcnNode {
    pub content_store: Arc<RwLock<ContentStore>>,
    pub pit: Arc<RwLock<PendingInterestTable>>,
    pub fib: Arc<RwLock<ForwardingInformationBase>>,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub coop_vm: Arc<RwLock<CoopVM>>,
    pub sharding_manager: Arc<RwLock<ShardingManager>>,
    pub execution_environment: Arc<RwLock<ExecutionEnvironment>>,
}

pub struct ExecutionEnvironment {
    pub state: String,
}

impl ExecutionEnvironment {
    pub fn new() -> Self {
        ExecutionEnvironment {
            state: String::new(),
        }
    }
}

impl IcnNode {
    pub fn new() -> Self {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let coop_vm = Arc::new(RwLock::new(CoopVM::new(Vec::new())));
        let sharding_manager = Arc::new(RwLock::new(ShardingManager::new(4, 10)));

        IcnNode {
            content_store: Arc::new(RwLock::new(ContentStore::new())),
            pit: Arc::new(RwLock::new(PendingInterestTable::new())),
            fib: Arc::new(RwLock::new(ForwardingInformationBase::new())),
            blockchain,
            coop_vm,
            sharding_manager,
            execution_environment: Arc::new(RwLock::new(ExecutionEnvironment::new())),
        }
    }

    pub fn process_cross_shard_transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut ShardingManager) -> Result<R>,
    {
        let mut sharding_manager = self.sharding_manager.write().map_err(|_| Error::ShardingError("Failed to acquire write lock on sharding manager".to_string()))?;
        f(&mut sharding_manager)
    }

    pub fn execute_smart_contract<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut ExecutionEnvironment) -> Result<R>,
    {
        let mut execution_environment = self.execution_environment.write().map_err(|_| Error::SmartContractError("Failed to acquire write lock on execution environment".to_string()))?;
        f(&mut execution_environment)
    }

    pub fn with_blockchain<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Blockchain) -> Result<R>,
    {
        let mut blockchain = self.blockchain.write().map_err(|_| Error::BlockchainError("Failed to acquire write lock on blockchain".to_string()))?;
        f(&mut blockchain)
    }

    pub fn with_coop_vm<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut CoopVM) -> Result<R>,
    {
        let mut coop_vm = self.coop_vm.write().map_err(|_| Error::VmError("Failed to acquire write lock on CoopVM".to_string()))?;
        f(&mut coop_vm)
    }

    pub fn with_sharding_manager<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&ShardingManager) -> Result<R>,
    {
        let sharding_manager = self.sharding_manager.read().map_err(|_| Error::ShardingError("Failed to acquire read lock on sharding manager".to_string()))?;
        f(&sharding_manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_currency::CurrencyType;
    use icn_blockchain::Transaction;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_cross_shard_transaction() {
        let node = IcnNode::new();

        // Initialize balances
        node.process_cross_shard_transaction(|sharding_manager| {
            sharding_manager.add_address_to_shard("Alice".to_string(), 0);
            sharding_manager.add_address_to_shard("Bob".to_string(), 1);
            sharding_manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0)
        }).unwrap();

        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            500.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        node.process_cross_shard_transaction(|sharding_manager| {
            sharding_manager.transfer_between_shards(0, 1, &transaction)
        }).unwrap();

        // Check balances after transaction
        let alice_balance = node.with_sharding_manager(|sharding_manager| {
            sharding_manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds)
        }).unwrap();
        let bob_balance = node.with_sharding_manager(|sharding_manager| {
            sharding_manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds)
        }).unwrap();

        assert_eq!(alice_balance, 500.0);
        assert_eq!(bob_balance, 500.0);
    }
}