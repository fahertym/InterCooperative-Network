// File: crates/icn_zkp/src/circuits.rs
use bulletproofs::r1cs::{ConstraintSystem, R1CSProof, Verifier};
use bulletproofs::{BulletproofGens, PedersenGens};
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use rand::thread_rng;

use icn_common::Transaction;

pub struct TransactionCircuit {
    amount: u64,
    balance: u64,
}

impl TransactionCircuit {
    pub fn new(transaction: &Transaction, balance: u64) -> Self {
        TransactionCircuit {
            amount: transaction.amount as u64,
            balance,
        }
    }

    pub fn prove(&self) -> (R1CSProof, Vec<Scalar>) {
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(128, 1);

        let (proof, commitments) = bulletproofs::r1cs::Prover::new(&pc_gens)
            .prove(
                &bp_gens,
                &mut Transcript::new(b"TransactionProof"),
                &|mut cs| {
                    let amount = cs.allocate_multiplier(self.amount.into())?;
                    let balance = cs.allocate_multiplier(self.balance.into())?;

                    cs.constrain(amount.0 - balance.0);

                    Ok(())
                },
                &mut thread_rng(),
            )
            .expect("Proof creation failed");

        (proof, commitments)
    }

    pub fn verify(proof: &R1CSProof, commitments: &[Scalar]) -> bool {
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(128, 1);

        let mut verifier_transcript = Transcript::new(b"TransactionProof");
        let mut verifier = Verifier::new(&mut verifier_transcript);

        let result = verifier
            .verify(
                &proof,
                &pc_gens,
                &bp_gens,
                &|mut cs| {
                    let amount = cs.allocate_multiplier(commitments[0])?;
                    let balance = cs.allocate_multiplier(commitments[1])?;

                    cs.constrain(amount.0 - balance.0);

                    Ok(())
                },
            )
            .is_ok();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::CurrencyType;

    #[test]
    fn test_transaction_proof() {
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 1234567890,
            signature: None,
        };

        let balance = 100;
        let circuit = TransactionCircuit::new(&transaction, balance);

        let (proof, commitments) = circuit.prove();
        assert!(TransactionCircuit::verify(&proof, &commitments));
    }
}