// crates/icn_utils/src/lib.rs

pub mod error;
pub mod types;
pub mod utils;

pub use error::{IcnError, IcnResult};
pub use types::{Block, Transaction, CurrencyType, Error, Result, Currency, CurrencySystem}; // Add necessary types from types module
pub use utils::*;
