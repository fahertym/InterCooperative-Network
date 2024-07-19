// crates/icn_core/src/lib.rs

use std::sync::{Arc, RwLock};

// Re-export key types from other crates
pub use icn_blockchain::{Block, Transaction, Blockchain};
pub use icn_consensus::PoCConsensus;
pub use icn_currency::CurrencyType;
pub use icn_governance::{DemocraticSystem, ProposalCategory, ProposalType};
pub use icn_identity::DecentralizedIdentity;
pub use icn_network::{Node as NetworkNode, Network};
pub use icn_node::{ContentStore, ForwardingInformationBase, PendingInterestTable};
pub use icn_sharding::ShardingManager;
pub use icn_vm::{CoopVM, Opcode};

mod error;
pub use error::{Error, Result};

pub struct IcnNode {
    pub content_store: Arc<RwLock<ContentStore>>,
    pub pit: Arc<RwLock<PendingInterestTable>>,
    pub fib: Arc<RwLock<ForwardingInformationBase>>,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub coop_vm: Arc<RwLock<CoopVM>>,
    pub sharding_manager: Arc<RwLock<ShardingManager>>,
}

impl IcnNode {
    pub fn new() -> Self {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let coop_vm = Arc::new(RwLock::new(CoopVM::new()));
        let sharding_manager = Arc::new(RwLock::new(ShardingManager::new(4, 10)));

        IcnNode {
            content_store: Arc::new(RwLock::new(ContentStore::new())),
            pit: Arc::new(RwLock::new(PendingInterestTable::new())),
            fib: Arc::new(RwLock::new(ForwardingInformationBase::new())),
            blockchain,
            coop_vm,
            sharding_manager,
        }
    }

    pub fn process_cross_shard_transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut ShardingManager) -> Result<R>,
    {
        let mut sharding_manager = self.sharding_manager.write().map_err(|_| Error::LockError("Failed to acquire write lock on sharding manager".to_string()))?;
        f(&mut sharding_manager)
    }

    pub fn with_blockchain<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Blockchain) -> Result<R>,
    {
        let mut blockchain = self.blockchain.write().map_err(|_| Error::LockError("Failed to acquire write lock on blockchain".to_string()))?;
        f(&mut blockchain)
    }

    pub fn with_coop_vm<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut CoopVM) -> Result<R>,
    {
        let mut coop_vm = self.coop_vm.write().map_err(|_| Error::LockError("Failed to acquire write lock on CoopVM".to_string()))?;
        f(&mut coop_vm)
    }

    pub fn with_sharding_manager<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&ShardingManager) -> Result<R>,
    {
        let sharding_manager = self.sharding_manager.read().map_err(|_| Error::LockError("Failed to acquire read lock on sharding manager".to_string()))?;
        f(&sharding_manager)
    }
}