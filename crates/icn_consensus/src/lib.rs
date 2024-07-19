use serde::{Serialize, Deserialize};
use icn_core::error::{Error, Result};
use log::{info, warn, debug};

mod bft_poc;
mod proof_of_cooperation;

pub use bft_poc::BFTPoC;
pub use proof_of_cooperation::ProofOfCooperation;

#[derive(Serialize, Deserialize)]
pub struct PoCConsensus {
    pub members: Vec<Member>,
    pub threshold: f64,
    pub quorum: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Member {
    pub id: String,
    pub reputation: f64,
    pub is_validator: bool,
}

impl PoCConsensus {
    pub fn new(threshold: f64, quorum: f64) -> Self {
        info!("Creating new PoCConsensus with threshold: {} and quorum: {}", threshold, quorum);
        PoCConsensus {
            members: Vec::new(),
            threshold,
            quorum,
        }
    }

    pub fn add_member(&mut self, member_id: String, is_validator: bool) -> Result<()> {
        debug!("Adding new member: {}, is_validator: {}", member_id, is_validator);
        if self.members.iter().any(|m| m.id == member_id) {
            return Err(Error::ConsensusError("Member already exists".to_string()));
        }
        self.members.push(Member { id: member_id.clone(), reputation: 1.0, is_validator });
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

    pub fn validate_block(&self, block_hash: &str, votes: &[(&str, bool)]) -> Result<bool> {
        debug!("Validating block: {}", block_hash);
        let total_reputation: f64 = self.members.iter().map(|m| m.reputation).sum();
        let mut positive_reputation = 0.0;
        let mut participating_reputation = 0.0;

        for (member_id, vote) in votes {
            if let Some(member) = self.members.iter().find(|m| &m.id == member_id) {
                participating_reputation += member.reputation;
                if *vote {
                    positive_reputation += member.reputation;
                }
            } else {
                return Err(Error::ConsensusError(format!("Invalid member in votes: {}", member_id)));
            }
        }

        if participating_reputation / total_reputation < self.quorum {
            return Err(Error::ConsensusError("Quorum not reached".to_string()));
        }

        Ok(positive_reputation / participating_reputation >= self.threshold)
    }

    pub fn update_reputation(&mut self, member_id: &str, change: f64) -> Result<()> {
        if let Some(member) = self.members.iter_mut().find(|m| m.id == member_id) {
            member.reputation += change;
            member.reputation = member.reputation.max(0.0);  // Ensure reputation doesn't go negative
            Ok(())
        } else {
            Err(Error::ConsensusError(format!("Member not found: {}", member_id)))
        }
    }

    pub fn get_total_reputation(&self) -> f64 {
        self.members.iter().map(|m| m.reputation).sum()
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
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        consensus.add_member("Alice".to_string(), true).unwrap();
        consensus.add_member("Bob".to_string(), true).unwrap();
        consensus.add_member("Charlie".to_string(), true).unwrap();

        // Update reputations
        consensus.update_reputation("Alice", 1.0).unwrap();
        consensus.update_reputation("Bob", 0.5).unwrap();

        let votes = vec![
            ("Alice", true),
            ("Bob", true),
            ("Charlie", false),
        ];

        assert!(consensus.validate_block("test_block_hash", &votes).unwrap());

        // Test with insufficient quorum
        let votes_insufficient = vec![
            ("Alice", true),
            ("Bob", false),
        ];

        assert!(consensus.validate_block("test_block_hash", &votes_insufficient).is_err());
    }

    #[test]
    fn test_update_reputation() {
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        consensus.add_member("Alice".to_string(), true).unwrap();

        consensus.update_reputation("Alice", 0.5).unwrap();
        assert_eq!(consensus.members[0].reputation, 1.5);

        consensus.update_reputation("Alice", -0.7).unwrap();
        assert_eq!(consensus.members[0].reputation, 0.8);

        // Ensure reputation doesn't go negative
        consensus.update_reputation("Alice", -1.0).unwrap();
        assert_eq!(consensus.members[0].reputation, 0.0);

        assert!(consensus.update_reputation("Bob", 1.0).is_err());
    }
}