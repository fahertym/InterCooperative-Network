// File: crates/icn_zkp/src/lib.rs

use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use rand::thread_rng;
use icn_common::{IcnResult, IcnError, Transaction};

pub struct ZKPManager {
    bp_gens: BulletproofGens,
    pc_gens: PedersenGens,
}

impl ZKPManager {
    pub fn new(max_bitsize: usize) -> Self {
        ZKPManager {
            bp_gens: BulletproofGens::new(max_bitsize, 1),
            pc_gens: PedersenGens::default(),
        }
    }

    pub fn create_proof(&self, transaction: &Transaction) -> IcnResult<(RangeProof, Vec<Scalar>)> {
        let amount = (transaction.amount * 100.0) as u64; // Convert to cents for integer representation
        let (proof, committed_value) = self.create_range_proof(amount)?;
        Ok((proof, vec![committed_value]))
    }

    pub fn verify_proof(&self, proof: &RangeProof, committed_values: &[Scalar]) -> IcnResult<bool> {
        if committed_values.len() != 1 {
            return Err(IcnError::ZKP("Invalid number of committed values".into()));
        }

        let mut transcript = Transcript::new(b"TransactionRangeProof");
        proof
            .verify_single(&self.bp_gens, &self.pc_gens, &mut transcript, &committed_values[0], 64)
            .map_err(|e| IcnError::ZKP(format!("Proof verification failed: {}", e)))
    }

    fn create_range_proof(&self, value: u64) -> IcnResult<(RangeProof, Scalar)> {
        let mut transcript = Transcript::new(b"TransactionRangeProof");
        let (proof, committed_value) = RangeProof::prove_single(
            &self.bp_gens,
            &self.pc_gens,
            &mut transcript,
            value,
            &Scalar::random(&mut thread_rng()),
            64,
        )
        .map_err(|e| IcnError::ZKP(format!("Failed to create range proof: {}", e)))?;

        Ok((proof, committed_value))
    }

    pub fn create_multi_proof(&self, values: &[u64]) -> IcnResult<(RangeProof, Vec<Scalar>)> {
        let mut transcript = Transcript::new(b"MultiRangeProof");
        let (proof, committed_values) = RangeProof::prove_multiple(
            &self.bp_gens,
            &self.pc_gens,
            &mut transcript,
            values,
            &vec![64; values.len()],
            &Scalar::random(&mut thread_rng()),
            &mut thread_rng(),
        )
        .map_err(|e| IcnError::ZKP(format!("Failed to create multi-range proof: {}", e)))?;

        Ok((proof, committed_values))
    }

    pub fn verify_multi_proof(&self, proof: &RangeProof, committed_values: &[Scalar]) -> IcnResult<bool> {
        let mut transcript = Transcript::new(b"MultiRangeProof");
        proof
            .verify_multiple(&self.bp_gens, &self.pc_gens, &mut transcript, committed_values, &vec![64; committed_values.len()])
            .map_err(|e| IcnError::ZKP(format!("Multi-proof verification failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::CurrencyType;

    #[test]
    fn test_create_and_verify_proof() {
        let zkp_manager = ZKPManager::new(64);
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 1234567890,
            signature: None,
        };

        let (proof, committed_value) = zkp_manager.create_proof(&transaction).unwrap();
        assert!(zkp_manager.verify_proof(&proof, &committed_value).unwrap());
    }

    #[test]
    fn test_invalid_proof() {
        let zkp_manager = ZKPManager::new(64);
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 1234567890,
            signature: None,
        };

        let (proof, mut committed_value) = zkp_manager.create_proof(&transaction).unwrap();
        committed_value[0] += Scalar::one(); // Tamper with the committed value
        assert!(!zkp_manager.verify_proof(&proof, &committed_value).unwrap());
    }

    #[test]
    fn test_multi_proof() {
        let zkp_manager = ZKPManager::new(64);
        let values = vec![100, 200, 300];

        let (proof, committed_values) = zkp_manager.create_multi_proof(&values).unwrap();
        assert!(zkp_manager.verify_multi_proof(&proof, &committed_values).unwrap());
    }
}