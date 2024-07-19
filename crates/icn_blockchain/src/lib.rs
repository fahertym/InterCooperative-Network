// crates/icn_blockchain/src/lib.rs

mod block;
mod transaction;
mod blockchain;

pub use block::Block;
pub use transaction::Transaction;
pub use blockchain::Blockchain;

use icn_currency::CurrencyType;
use icn_consensus::PoCConsensus;

// Re-export the Error and Result types
mod error;
pub use error::{Error, Result};