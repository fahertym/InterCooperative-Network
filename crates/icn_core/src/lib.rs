use std::sync::{Arc, RwLock};

// Re-export key types from other crates
#[cfg(feature = "blockchain")]
pub use icn_blockchain::{Block, Transaction, Blockchain};
#[cfg(feature = "currency")]
pub use icn_currency::CurrencyType;
#[cfg(feature = "governance")]
pub use icn_governance::{DemocraticSystem, ProposalCategory, ProposalType};
#[cfg(feature = "identity")]
pub use icn_identity::DecentralizedIdentity;
#[cfg(feature = "network")]
pub use icn_network::{Node as NetworkNode, Network, Packet, PacketType};
#[cfg(feature = "sharding")]
pub use icn_sharding::ShardingManager;
#[cfg(feature = "vm")]
pub use icn_vm::{CoopVM, Opcode};

pub mod error;
pub use error::{Error, Result};

pub mod cli;
pub mod logging;
pub mod security;

pub struct IcnNode {
    // Add fields as needed
}

impl IcnNode {
    pub fn new() -> Self {
        IcnNode {
            // Initialize fields
        }
    }

    // Add methods as needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icn_node_creation() {
        let node = IcnNode::new();
        // Add assertions as needed
    }
}