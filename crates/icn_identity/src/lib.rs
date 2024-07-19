mod did;
mod identity_manager;

pub use did::DecentralizedIdentity;
pub use identity_manager::IdentityManager;

use icn_core::error::{Error, Result};