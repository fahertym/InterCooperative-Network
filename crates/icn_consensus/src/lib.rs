// InterCooperative-Network/crates/icn_consensus/src/lib.rs

mod bft_poc;
mod proof_of_cooperation;

use serde::{Serialize, Deserialize};
use icn_core::error::{Error, Result};
use log::{info, warn, debug};

#[derive(Serialize, Deserialize)]
pub struct PoCConsensus {
    pub members: Vec<Member>,
    pub threshold: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub is_validator: bool,
}

impl PoCConsensus {
    pub fn new(threshold: f64, _quorum: f64) -> Self {
        info!("Creating new PoCConsensus with threshold: {}", threshold);
        PoCConsensus {
            members: Vec::new(),
            threshold,
        }
    }

    pub fn add_member(&mut self, member_id: String, is_validator: bool) -> Result<()> {
        debug!("Adding new member: {}, is_validator: {}", member_id, is_validator);
        self.members.push(Member { id: member_id.clone(), is_validator });
        info!("Added new member: {}", member_id);
        Ok(())
    }

    pub fn remove_member(&mut self, member_id: &str) -> Result<()> {
        debug!("Attempting to remove member: {}", member_id);
        if let Some(index) = self.members.iter().position(|m| m.id == member_id) {
            self.members.remove(index);
            info!("Removed member: {}", member_id);
            Ok(())
        } else {
            warn!("Failed to remove member: {} (not found)", member_id);
            Err(Error::ConsensusError(format!("Member not found: {}", member_id)))
        }
    }

    pub fn is_validator(&self, member_id: &str) -> bool {
        let is_validator = self.members.iter().any(|m| m.id == member_id && m.is_validator);
        debug!("Checking if {} is a validator: {}", member_id, is_validator);
        is_validator
    }

    pub fn validate_block(&self, _block_hash: &str) -> Result<bool> {
        debug!("Validating block: {}", _block_hash);
        // Implement the actual validation logic here
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_remove_member() {
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        
        assert!(consensus.add_member("Alice".to_string(), true).is_ok());
        assert_eq!(consensus.members.len(), 1);
        assert!(consensus.is_validator("Alice"));

        assert!(consensus.add_member("Bob".to_string(), false).is_ok());
        assert_eq!(consensus.members.len(), 2);
        assert!(!consensus.is_validator("Bob"));

        assert!(consensus.remove_member("Alice").is_ok());
        assert_eq!(consensus.members.len(), 1);
        assert!(!consensus.is_validator("Alice"));

        assert!(consensus.remove_member("Charlie").is_err());
    }

    #[test]
    fn test_validate_block() {
        let consensus = PoCConsensus::new(0.5, 0.66);
        assert!(consensus.validate_block("test_block_hash").unwrap());
    }
}