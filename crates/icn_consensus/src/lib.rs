// crates/icn_consensus/src/lib.rs

use serde::{Serialize, Deserialize};
use log::error;
use icn_common::{Error, Result};
use std::collections::HashMap;

mod proof_of_cooperation;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Member {
    pub id: String,
    pub reputation: f64,
    pub is_validator: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoCConsensus {
    pub members: HashMap<String, Member>,
    pub threshold: f64,
    pub quorum: f64,
}

impl PoCConsensus {
    pub fn new() -> Self {
        PoCConsensus {
            members: HashMap::new(),
            threshold: 0.66, // 66% agreement required for consensus
            quorum: 0.51, // 51% participation required for quorum
        }
    }

    pub fn add_member(&mut self, id: String, is_validator: bool) -> Result<()> {
        if self.members.contains_key(&id) {
            return Err(Error { message: "Member already exists".to_string() });
        }
        self.members.insert(id.clone(), Member {
            id,
            reputation: 1.0,
            is_validator,
        });
        Ok(())
    }

    pub fn remove_member(&mut self, id: &str) -> Result<()> {
        if self.members.remove(id).is_none() {
            return Err(Error { message: "Member not found".to_string() });
        }
        Ok(())
    }

    pub fn update_reputation(&mut self, id: &str, change: f64) -> Result<()> {
        let member = self.members.get_mut(id)
            .ok_or_else(|| Error { message: "Member not found".to_string() })?;
        member.reputation += change;
        member.reputation = member.reputation.max(0.0); // Ensure reputation doesn't go negative
        Ok(())
    }

    pub fn validate_block(&self, _block_hash: &str, votes: &[(&str, bool)]) -> Result<bool> {
        let total_reputation: f64 = self.members.values()
            .filter(|m| m.is_validator)
            .map(|m| m.reputation)
            .sum();

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
                return Err(Error { message: "Invalid member in votes".to_string() });
            }
        }

        if participating_reputation / total_reputation < self.quorum {
            return Err(Error { message: "Quorum not reached".to_string() });
        }

        Ok(positive_reputation / participating_reputation >= self.threshold)
    }

    pub fn get_validators(&self) -> Vec<&Member> {
        self.members.values().filter(|m| m.is_validator).collect()
    }
}

// crates/icn_consensus/src/proof_of_cooperation.rs

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofOfCooperation {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: String,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, timestamp: u64, data: String) -> Self {
        let mut block = Block {
            index,
            previous_hash,
            timestamp,
            data,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let input = format!("{}{}{}{}", self.index, self.previous_hash, self.timestamp, self.data);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl ProofOfCooperation {
    pub fn new() -> Self {
        ProofOfCooperation {
            blocks: Vec::new(),
        }
    }

    pub fn add_block(&mut self, data: String) -> Result<(), String> {
        let previous_hash = if let Some(last_block) = self.blocks.last() {
            last_block.hash.clone()
        } else {
            String::new()
        };

        let new_block = Block::new(
            self.blocks.len() as u64,
            previous_hash,
            chrono::Utc::now().timestamp_millis() as u64,
            data,
        );

        self.blocks.push(new_block);
        Ok(())
    }

    pub fn validate_chain(&self) -> Result<(), String> {
        for i in 1..self.blocks.len() {
            let previous_block = &self.blocks[i - 1];
            let current_block = &self.blocks[i];

            if current_block.previous_hash != previous_block.hash {
                return Err("Invalid previous hash".to_string());
            }

            if current_block.hash != current_block.calculate_hash() {
                return Err("Invalid block hash".to_string());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_remove_member() {
        let mut consensus = PoCConsensus::new();
        assert!(consensus.add_member("Alice".to_string(), true).is_ok());
        assert!(consensus.add_member("Bob".to_string(), false).is_ok());
        assert_eq!(consensus.members.len(), 2);
        assert!(consensus.remove_member("Alice").is_ok());
        assert_eq!(consensus.members.len(), 1);
        assert!(consensus.remove_member("Charlie").is_err());
    }

    #[test]
    fn test_update_reputation() {
        let mut consensus = PoCConsensus::new();
        consensus.add_member("Alice".to_string(), true).unwrap();
        assert!(consensus.update_reputation("Alice", 0.5).is_ok());
        assert_eq!(consensus.members.get("Alice").unwrap().reputation, 1.5);
        assert!(consensus.update_reputation("Bob", 1.0).is_err());
    }

    #[test]
    fn test_validate_block() {
        let mut consensus = PoCConsensus::new();
        consensus.add_member("Alice".to_string(), true).unwrap();
        consensus.add_member("Bob".to_string(), true).unwrap();
        consensus.add_member("Charlie".to_string(), true).unwrap();

        let votes = vec![
            ("Alice", true),
            ("Bob", true),
            ("Charlie", false),
        ];

        assert!(consensus.validate_block("block_hash", &votes).unwrap());

        let insufficient_votes = vec![
            ("Alice", true),