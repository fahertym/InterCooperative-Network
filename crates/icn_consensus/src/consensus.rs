use icn_common::{Block, IcnResult, IcnError};
use std::collections::HashMap;

pub struct PoCConsensus {
    threshold: f64,
    quorum: f64,
    validators: HashMap<String, f64>, // validator_id -> reputation
}

impl PoCConsensus {
    pub fn new(threshold: f64, quorum: f64) -> IcnResult<Self> {
        if threshold <= 0.0 || threshold > 1.0 || quorum <= 0.0 || quorum > 1.0 {
            return Err(IcnError::Consensus("Invalid threshold or quorum value".into()));
        }

        Ok(PoCConsensus {
            threshold,
            quorum,
            validators: HashMap::new(),
        })
    }

    pub fn add_validator(&mut self, validator_id: String, initial_reputation: f64) -> IcnResult<()> {
        if initial_reputation < 0.0 || initial_reputation > 1.0 {
            return Err(IcnError::Consensus("Invalid initial reputation".into()));
        }
        self.validators.insert(validator_id, initial_reputation);
        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> IcnResult<bool> {
        // In a real implementation, this would involve more complex validation logic
        // For now, we'll just check if the block has transactions
        if block.transactions.is_empty() {
            return Err(IcnError::Consensus("Block has no transactions".into()));
        }
        Ok(true)
    }

    pub fn update_reputation(&mut self, validator_id: &str, change: f64) -> IcnResult<()> {
        let reputation = self.validators.get_mut(validator_id)
            .ok_or_else(|| IcnError::Consensus("Validator not found".into()))?;
        *reputation += change;
        *reputation = reputation.clamp(0.0, 1.0);
        Ok(())
    }

    pub fn get_total_reputation(&self) -> f64 {
        self.validators.values().sum()
    }

    pub fn is_quorum_reached(&self, participating_reputation: f64) -> bool {
        let total_reputation = self.get_total_reputation();
        participating_reputation / total_reputation >= self.quorum
    }

    pub fn is_consensus_reached(&self, approving_reputation: f64, participating_reputation: f64) -> bool {
        approving_reputation / participating_reputation >= self.threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_creation() {
        assert!(PoCConsensus::new(0.66, 0.51).is_ok());
        assert!(PoCConsensus::new(1.5, 0.5).is_err());
        assert!(PoCConsensus::new(0.5, 1.5).is_err());
    }

    #[test]
    fn test_add_validator() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        assert!(consensus.add_validator("validator1".to_string(), 0.8).is_ok());
        assert!(consensus.add_validator("validator2".to_string(), 1.2).is_err());
    }

    #[test]
    fn test_validate_block() {
        let consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        let valid_block = Block {
            index: 1,
            timestamp: 12345,
            transactions: vec![icn_common::Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 50.0,
                currency_type: icn_common::CurrencyType::BasicNeeds,
                timestamp: 12345,
                signature: None,
            }],
            previous_hash: "previous_hash".to_string(),
            hash: "current_hash".to_string(),
        };
        assert!(consensus.validate_block(&valid_block).unwrap());

        let invalid_block = Block {
            index: 2,
            timestamp: 23456,
            transactions: vec![],
            previous_hash: "previous_hash".to_string(),
            hash: "current_hash".to_string(),
        };
        assert!(consensus.validate_block(&invalid_block).is_err());
    }

    #[test]
    fn test_reputation_update() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_validator("validator1".to_string(), 0.5).unwrap();
        
        assert!(consensus.update_reputation("validator1", 0.2).is_ok());
        assert_eq!(consensus.validators["validator1"], 0.7);

        // Test upper bound
        assert!(consensus.update_reputation("validator1", 0.5).is_ok());
        assert_eq!(consensus.validators["validator1"], 1.0);

        // Test lower bound
        assert!(consensus.update_reputation("validator1", -1.5).is_ok());
        assert_eq!(consensus.validators["validator1"], 0.0);

        // Test updating non-existent validator
        assert!(consensus.update_reputation("validator2", 0.1).is_err());
    }

    #[test]
    fn test_quorum_and_consensus() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_validator("validator1".to_string(), 0.3).unwrap();
        consensus.add_validator("validator2".to_string(), 0.3).unwrap();
        consensus.add_validator("validator3".to_string(), 0.3).unwrap();

        // Test quorum
        assert!(!consensus.is_quorum_reached(0.4));
        assert!(consensus.is_quorum_reached(0.5));

        // Test consensus
        assert!(!consensus.is_consensus_reached(0.3, 0.6));
        assert!(consensus.is_consensus_reached(0.4, 0.6));
    }

    #[test]
    fn test_total_reputation() {
        let mut consensus = PoCConsensus::new(0.66, 0.51).unwrap();
        consensus.add_validator("validator1".to_string(), 0.3).unwrap();
        consensus.add_validator("validator2".to_string(), 0.4).unwrap();
        consensus.add_validator("validator3".to_string(), 0.2).unwrap();

        assert_eq!(consensus.get_total_reputation(), 0.9);

        consensus.update_reputation("validator2", 0.1).unwrap();
        assert_eq!(consensus.get_total_reputation(), 1.0);
    }
}