// File: crates/icn_market/src/lib.rs

pub mod entities;
pub mod market;
pub mod transaction;
pub mod order_book;
pub mod matching_engine;
pub mod market_maker;
pub mod price_discovery;
pub mod risk_management;
pub mod analytics;

pub use entities::*;
pub use market::*;
pub use transaction::*;
pub use order_book::*;
pub use matching_engine::*;
pub use market_maker::*;
pub use price_discovery::*;
pub use risk_management::*;
pub use analytics::*;