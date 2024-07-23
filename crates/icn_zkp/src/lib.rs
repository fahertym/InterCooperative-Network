// File: icn_zkp/src/lib.rs

use bellman::{Circuit, ConstraintSystem, SynthesisError};
use bls12_381::Bls12;
use icn_common::{IcnResult, IcnError, Transaction};
use rand::rngs::OsRng;

mod circuits;
pub use circuits::*;

pub struct ZKPManager {
    params: bellman::groth16::Parameters<Bls12>,
    pvk: bellman::groth16::PreparedVerifyingKey<Bls12>,
}

impl ZKPManager {
    /// Creates a new ZKPManager with generated parameters.
    pub fn new() -> IcnResult<Self> {
        let mut rng = OsRng;
        let params = bellman::groth16::generate_random_parameters::<Bls12, _, _>(
            TransactionCircuit::empty(),
            &mut rng
        ).map_err(|e| IcnError::ZKP(format!("Failed to generate ZKP parameters: {}", e)))?;

        let pvk = bellman::groth16::prepare_verifying_key(&params.vk);

        Ok(ZKPManager { params, pvk })
    }

    /// Creates a zero-knowledge proof for a given transaction.
    pub fn create_proof(&self, transaction: &Transaction) -> IcnResult<bellman::groth16::Proof<Bls12>> {
        let circuit = TransactionCircuit::new(transaction);
        let mut rng = OsRng;

        bellman::groth16::create_random_proof(circuit, &self.params, &mut rng)
            .map_err(|e| IcnError::ZKP(format!("Failed to create ZKP proof: {}", e)))
    }

    /// Verifies a zero-knowledge proof for a given transaction.
    pub fn verify_proof(
        &self,
        proof: &bellman::groth16::Proof<Bls12>,
        transaction: &Transaction,
    ) -> IcnResult<bool> {
        let inputs = TransactionCircuit::public_inputs(transaction);

        bellman::groth16::verify_proof(&self.pvk, proof, &inputs)
            .map_err(|e| IcnError::ZKP(format!("Failed to verify ZKP proof: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::CurrencyType;

    #[test]
    fn test_zkp_create_and_verify() {
        let manager = ZKPManager::new().expect("Failed to create ZKPManager");

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 1234567890,
            signature: None,
        };

        let proof = manager.create_proof(&transaction).expect("Failed to create proof");
        let is_valid = manager.verify_proof(&proof, &transaction).expect("Failed to verify proof");

        assert!(is_valid, "Proof verification failed");
    }
}