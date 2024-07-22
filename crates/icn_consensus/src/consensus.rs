// icn_consensus/src/consensus.rs
use icn_common::{IcnResult, IcnError, Block, Transaction, CurrencyType};
use std::collections::HashMap;
use log::{info, warn};
use rand::Rng;

/// Represents a member of the consensus network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub reputation: f64,
    pub is_validator: bool,
}

/// Proof of Contribution (PoC) consensus mechanism.
#[derive(Debug, Serialize, Deserialize)]
pub struct PoCConsensus {
    pub members: HashMap<String, Member>,
    pub threshold: f64,
    pub quorum: f64,
}

impl PoCConsensus {
    /// Creates a new PoCConsensus with the specified threshold and quorum.
    ///
    /// # Arguments
    ///
    /// * `threshold` - The threshold for consensus.
    /// * `quorum` - The quorum for consensus.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Consensus` if the threshold or quorum values are invalid.
    pub fn new(threshold: f64, quorum: f64) -> IcnResult<Self> {
        if threshold <= 0.0 || threshold > 1.0 || quorum <= 0.0 || quorum > 1.0 {
            return Err(IcnError::Consensus("Invalid threshold or quorum value".into()));
        }

        Ok(PoCConsensus {
            members: HashMap::new(),
            threshold,
            quorum,
        })
    }

    /// Adds a new member to the consensus network.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the new member.
    /// * `is_validator` - Whether the new member is a validator.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Consensus` if the member already exists.
    pub fn add_member(&mut self, id: String, is_validator: bool) -> IcnResult<()> {
        if self.members.contains_key(&id) {
            return Err(IcnError::Consensus("Member already exists".into()));
        }
        self.members.insert(id.clone(), Member {
            id,
            reputation: 1.0,
            is_validator,
        });
        Ok(())
    }

    /// Removes a member from the consensus network.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the member to remove.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Consensus` if the member is not found.
    pub fn remove_member(&mut self, id: &str) -> IcnResult<()> {
        if self.members.remove(id).is_none() {
            return Err(IcnError::Consensus("Member not found".into()));
        }
        Ok(())
    }

    /// Updates the reputation of a member.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the member.
    /// * `change` - The amount to change the reputation by.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Consensus` if the member is not found.
    pub fn update_reputation(&mut self, id: &str, change: f64) -> IcnResult<()> {
        let member = self.members.get_mut(id).ok_or_else(|| IcnError::Consensus("Member not found".into()))?;
        member.reputation += change;
        member.reputation = member.reputation.max(0.0); // Ensure reputation doesn't go negative
        Ok(())
    }

    /// Validates a block.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to validate.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Consensus` if the block is invalid.
    pub fn validate_block(&self, block: &Block) -> IcnResult<bool> {
        // In a real implementation, this would involve more complex validation logic
        if block.transactions.is_empty() {
            return Err(IcnError::Consensus("Block has no transactions".into()));
        }
        Ok(true)
    }

    /// Reaches consensus on a block.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to reach consensus on.
    /// * `votes` - The votes for the block.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Consensus` if consensus is not reached.
    pub fn reach_consensus(&self, block: &Block, votes: &[(&str, bool)]) -> IcnResult<bool> {
        let total_reputation: f64 = self.members.values().filter(|m| m.is_validator).map(|m| m.reputation).sum();

        let mut positive_reputation = 0.0;
        let mut participating_reputation = 0.0;

        for (member_id, vote) in votes {
            if let Some(member) = self.members.get(*member_id) {
                if member.is_validator {
                    participating_reputation += member.reputation;
                    if *vote {
                        positive_reputation += member.reputation;
                    }
                }
            } else {
                return Err(IcnError::Consensus("Invalid member in votes".into()));
            }
        }

        if participating_reputation / total_reputation < self.quorum {
            return Err(IcnError::Consensus("Quorum not reached".into()));
        }

        Ok(positive_reputation / participating_reputation >= self.threshold)
    }

    /// Returns a list of all validators.
    pub fn get_validators(&self) -> Vec<&Member> {
        self.members.values().filter(|m| m.is_validator).collect()
    }

    /// Selects a proposer for the next block.
    ///
    /// # Errors
    ///
    /// Returns `IcnError::Consensus` if no validators are available.
    pub fn select_proposer(&self) -> IcnResult<&Member> {
        let validators: Vec<&Member> = self.get_validators();
        if validators.is_empty() {
            return Err(IcnError::Consensus("No validators available".into()));
        }

        let total_reputation: f64 = validators.iter().map(|m| m.reputation).sum();
        let mut rng = rand::thread_rng();
        let random_point = rng.gen::<f64>() * total_reputation;

        let mut cumulative_reputation = 0.0;
        for validator in &validators {
            cumulative_reputation += validator.reputation;
            if cumulative_reputation > random_point {
                return Ok(validator);
            }
        }

        // This should never happen, but we'll return the last validator if it does
        Ok(validators.last().unwrap())
    }

    /// Starts the PoC consensus mechanism.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn start(&self) -> IcnResult<()> {
        info!("PoC Consensus mechanism started");
        Ok(())
    }

    /// Stops the PoC consensus mechanism.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn stop(&self) -> IcnResult<()> {
        info!("PoC Consensus mechanism stopped");
        Ok(())
    }
}

/// Trait defining the methods required for a consensus algorithm.
pub trait ConsensusAlgorithm {
    fn validate_block(&self, block: &Block) -> IcnResult<bool>;
    fn reach_consensus(&self, block: &Block, votes: &[(&str, bool)]) -> IcnResult<bool>;
}

impl ConsensusAlgorithm for PoCConsensus {
    fn validate_block(&self, block: &Block) -> IcnResult<bool> {
        self.validate_block(block)
    }

    fn reach_consensus(&self, block: &Block, votes: &[(&str, bool)]) -> IcnResult<bool> {
        self.reach_consensus(block, votes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::CurrencyType;

    #[test]
    fn test_add_and_remove_member() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        assert!(consensus.add_member("Alice".to_string(), true).is_ok());
        assert!(consensus.add_member("Bob".to_string(), false).is_ok());
        assert_eq!(consensus.members.len(), 2);
        assert!(consensus.remove_member("Alice").is_ok());
        assert_eq!(consensus.members.len(), 1);
        assert!(consensus.remove_member("Charlie").is_err());
    }

    #[test]
    fn test_update_reputation() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_member("Alice".to_string(), true).unwrap();
        assert!(consensus.update_reputation("Alice", 0.5).is_ok());
        assert_eq!(consensus.members.get("Alice").unwrap().reputation, 1.5);
        assert!(consensus.update_reputation("Bob", 1.0).is_err());
    }

    #[test]
    fn test_reach_consensus() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_member("Alice".to_string(), true).unwrap();
        consensus.add_member("Bob".to_string(), true).unwrap();
        consensus.add_member("Charlie".to_string(), true).unwrap();

        let block = Block {
            index: 1,
            timestamp: chrono::Utc::now().timestamp(),
            transactions: vec![Transaction::new(
                "Alice".to_string(),
                "Bob".to_string(),
                100.0,
                CurrencyType::BasicNeeds,
                1000,
            )],
            previous_hash: "previous_hash".to_string(),
            hash: "hash".to_string(),
        };

        let votes = vec![
            ("Alice", true),
            ("Bob", true),
            ("Charlie", false),
        ];

        assert!(consensus.reach_consensus(&block, &votes).unwrap());

        let insufficient_votes = vec![
            ("Alice", true),
            ("Bob", true),
        ];

        assert!(consensus.reach_consensus(&block, &insufficient_votes).is_err());
    }

    #[test]
    fn test_select_proposer() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_member("Alice".to_string(), true).unwrap();
        consensus.add_member("Bob".to_string(), true).unwrap();
        consensus.add_member("Charlie".to_string(), false).unwrap();

        let proposer = consensus.select_proposer().unwrap();
        assert!(proposer.is_validator);
        assert!(proposer.id == "Alice" || proposer.id == "Bob");

        // Test when there are no validators
        let mut consensus_no_validators = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus_no_validators.add_member("Dave".to_string(), false).unwrap();
        assert!(consensus_no_validators.select_proposer().is_err());
    }
}
