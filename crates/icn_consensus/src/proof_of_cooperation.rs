use crate::PoCConsensus;
use icn_core::error::{Error, Result};
use std::collections::HashMap;
use log::{info, warn, debug};
use sha2::{Sha256, Digest};

pub struct ProofOfCooperation {
    consensus: PoCConsensus,
    cooperation_proofs: HashMap<String, CooperationProof>,
}

struct CooperationProof {
    member_id: String,
    proof: Vec<u8>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl ProofOfCooperation {
    pub fn new(threshold: f64, quorum: f64) -> Self {
        ProofOfCooperation {
            consensus: PoCConsensus::new(threshold, quorum),
            cooperation_proofs: HashMap::new(),
        }
    }

    pub fn submit_cooperation_proof(&mut self, member_id: &str, proof: Vec<u8>) -> Result<()> {
        debug!("Submitting cooperation proof for member: {}", member_id);
        if !self.consensus.is_validator(member_id) {
            return Err(Error::ConsensusError("Member is not a validator".to_string()));
        }

        let cooperation_proof = CooperationProof {
            member_id: member_id.to_string(),
            proof,
            timestamp: chrono::Utc::now(),
        };

        self.cooperation_proofs.insert(member_id.to_string(), cooperation_proof);
        info!("Cooperation proof submitted for member: {}", member_id);
        Ok(())
    }

    pub fn validate_cooperation(&self, member_id: &str) -> Result<bool> {
        debug!("Validating cooperation for member: {}", member_id);
        let proof = self.cooperation_proofs.get(member_id)
            .ok_or_else(|| Error::ConsensusError("No cooperation proof found".to_string()))?;

        // In a real implementation, this would involve more complex validation logic
        // For this example, we'll use a simple hash-based validation
        let mut hasher = Sha256::new();
        hasher.update(&proof.proof);
        let hash = hasher.finalize();

        // Consider cooperation valid if the hash starts with a zero byte
        let is_valid = hash[0] == 0;

        if is_valid {
            info!("Cooperation proof valid for member: {}", member_id);
        } else {
            warn!("Cooperation proof invalid for member: {}", member_id);
        }

        Ok(is_valid)
    }

    pub fn update_reputation(&mut self, member_id: &str) -> Result<()> {
        debug!("Updating reputation for member: {}", member_id);
        let is_cooperative = self.validate_cooperation(member_id)?;

        let reputation_change = if is_cooperative { 0.1 } else { -0.1 };
        self.consensus.update_reputation(member_id, reputation_change)?;

        info!("Reputation updated for member: {}", member_id);
        Ok(())
    }

    pub fn get_member_reputation(&self, member_id: &str) -> Result<f64> {
        self.consensus.members.iter()
            .find(|m| m.id == member_id)
            .map(|m| m.reputation)
            .ok_or_else(|| Error::ConsensusError("Member not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn setup_poc() -> ProofOfCooperation {
        let mut poc = ProofOfCooperation::new(0.5, 0.66);
        poc.consensus.add_member("Alice".to_string(), true).unwrap();
        poc.consensus.add_member("Bob".to_string(), true).unwrap();
        poc.consensus.add_member("Charlie".to_string(), true).unwrap();
        poc
    }

    fn generate_random_proof() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let proof: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        proof
    }

    #[test]
    fn test_submit_cooperation_proof() {
        let mut poc = setup_poc();
        let proof = generate_random_proof();

        assert!(poc.submit_cooperation_proof("Alice", proof.clone()).is_ok());
        assert!(poc.cooperation_proofs.contains_key("Alice"));
        assert_eq!(poc.cooperation_proofs.get("Alice").unwrap().proof, proof);
    }

    #[test]
    fn test_validate_cooperation() {
        let mut poc = setup_poc();
        let proof = vec![0; 32]; // This will always be considered valid in our simple implementation

        poc.submit_cooperation_proof("Alice", proof).unwrap();
        assert!(poc.validate_cooperation("Alice").unwrap());

        let invalid_proof = vec![1; 32]; // This will always be considered invalid
        poc.submit_cooperation_proof("Bob", invalid_proof).unwrap();
        assert!(!poc.validate_cooperation("Bob").unwrap());
    }

    #[test]
    fn test_update_reputation() {
        let mut poc = setup_poc();
        let valid_proof = vec![0; 32];
        let invalid_proof = vec![1; 32];

        poc.submit_cooperation_proof("Alice", valid_proof).unwrap();
        poc.submit_cooperation_proof("Bob", invalid_proof).unwrap();

        poc.update_reputation("Alice").unwrap();
        poc.update_reputation("Bob").unwrap();

        assert!(poc.get_member_reputation("Alice").unwrap() > 1.0);
        assert!(poc.get_member_reputation("Bob").unwrap() < 1.0);
    }

    #[test]
    fn test_get_member_reputation() {
        let poc = setup_poc();

        assert_eq!(poc.get_member_reputation("Alice").unwrap(), 1.0);
        assert!(poc.get_member_reputation("NonExistentMember").is_err());
    }

    #[test]
    fn test_invalid_member_submission() {
        let mut poc = setup_poc();
        let proof = generate_random_proof();

        assert!(poc.submit_cooperation_proof("NonExistentMember", proof).is_err());
    }

    #[test]
    fn test_validate_nonexistent_cooperation() {
        let poc = setup_poc();

        assert!(poc.validate_cooperation("Alice").is_err());
    }

    #[test]
    fn test_update_reputation_without_proof() {
        let mut poc = setup_poc();

        assert!(poc.update_reputation("Alice").is_err());
    }

    #[test]
    fn test_multiple_reputation_updates() {
        let mut poc = setup_poc();
        let valid_proof = vec![0; 32];

        poc.submit_cooperation_proof("Alice", valid_proof.clone()).unwrap();
        
        for _ in 0..5 {
            poc.update_reputation("Alice").unwrap();
        }

        let final_reputation = poc.get_member_reputation("Alice").unwrap();
        assert!(final_reputation > 1.4 && final_reputation < 1.6);
    }
}