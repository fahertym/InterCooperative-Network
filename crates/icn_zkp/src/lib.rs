use bellman::{Circuit, ConstraintSystem, SynthesisError};
use bls12_381::Bls12;
use icn_common::{IcnResult, IcnError, Transaction};
use rand::rngs::OsRng;

mod circuits;
pub use circuits::*;

pub struct ZKPManager {
    params: bellman::groth16::Parameters<Bls12>,
}

impl ZKPManager {
    pub fn new() -> IcnResult<Self> {
        let mut rng = OsRng;
        let params = bellman::groth16::generate_random_parameters::<Bls12, _, _>(
            circuits::TransactionCircuit::empty(),
            &mut rng
        ).map_err(|e| IcnError::ZKP(format!("Failed to generate ZKP parameters: {}", e)))?;

        Ok(ZKPManager { params })
    }

    pub fn create_proof<C>(&self, circuit: C) -> IcnResult<bellman::groth16::Proof<Bls12>>
    where
        C: Circuit<bls12_381::Scalar>,
    {
        let mut rng = OsRng;
        bellman::groth16::create_random_proof