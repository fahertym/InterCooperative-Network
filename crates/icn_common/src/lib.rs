/// Module for common types and utilities used across the ICN project.
use chrono::{DateTime, Utc};
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;
use rand::RngCore;
use ed25519_dalek::Keypair;
use bellman::{Circuit, ConstraintSystem, SynthesisError};
use bls12_381::Bls12;

pub mod error;
pub use error::{IcnError, IcnResult};

/// Enumeration representing different types of currencies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CurrencyType {
    BasicNeeds,
    Education,
    Environmental,
    Community,
    Volunteer,
    Storage,
    Processing,
    Energy,
    Luxury,
    Service,
    Custom(String),
    AssetToken(String),
    Bond(String),
}

/// Structure representing a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: i64,
    pub signature: Option<Vec<u8>>,
    pub zkp: Option<ZKProof>,
}

/// Structure representing a block in the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub zkp_accumulator: Option<ZKAccumulator>,
}

/// Structure representing a proposal for governance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub status: ProposalStatus,
    pub proposal_type: ProposalType,
    pub category: ProposalCategory,
    pub required_quorum: f64,
    pub execution_timestamp: Option<DateTime<Utc>>,
}

/// Enumeration representing the status of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Implemented,
}

/// Enumeration representing the type of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
}

/// Enumeration representing the category of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalCategory {
    Constitutional,
    Economic,
    Technical,
}

/// Structure representing a vote on a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
    pub zkp: Option<ZKProof>,
}

/// Trait defining an object that can be hashed.
pub trait Hashable {
    fn hash(&self) -> String;
}

/// Structure representing a zero-knowledge proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKProof {
    pub proof: Vec<u8>,
    pub public_inputs: Vec<Vec<u8>>,
}

/// Structure representing a zero-knowledge proof accumulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKAccumulator {
    pub value: Vec<u8>,
}

/// Trait for synthesizing zero-knowledge circuits.
pub trait ZKCircuit<E: bellman::Engine> {
    fn synthesize<CS: ConstraintSystem<E>>(self, cs: &mut CS) -> Result<(), SynthesisError>;
}

/// Implementation of the Hashable trait for Block.
impl Hashable for Block {
    fn hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(&self.timestamp.to_string());
        for transaction in &self.transactions {
            hasher.update(&transaction.hash());
        }
        hasher.update(&self.previous_hash);
        format!("{:x}", hasher.finalize())
    }
}

/// Implementation of the Hashable trait for Transaction.
impl Hashable for Transaction {
    fn hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.from);
        hasher.update(&self.to);
        hasher.update(self.amount.to_string().as_bytes());
        hasher.update(format!("{:?}", self.currency_type).as_bytes());
        hasher.update(self.timestamp.to_string().as_bytes());
        if let Some(signature) = &self.signature {
            hasher.update(signature);
        }
        if let Some(zkp) = &self.zkp {
            hasher.update(&zkp.proof);
            for input in &zkp.public_inputs {
                hasher.update(input);
            }
        }
        format!("{:x}", hasher.finalize())
    }
}

/// Implementation of methods for Transaction.
impl Transaction {
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType, timestamp: i64) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            timestamp,
            signature: None,
            zkp: None,
        }
    }

    pub fn sign(&mut self, private_key: &[u8]) -> IcnResult<()> {
        use ed25519_dalek::{Keypair, Signer};
        let keypair = Keypair::from_bytes(private_key)
            .map_err(|e| IcnError::Identity(format!("Invalid private key: {}", e)))?;
        let message = self.hash().as_bytes().to_vec();
        let signature = keypair.sign(&message);
        self.signature = Some(signature.to_bytes().to_vec());
        Ok(())
    }

    pub fn verify(&self, public_key: &[u8]) -> IcnResult<bool> {
        use ed25519_dalek::{PublicKey, Verifier};
        let public_key = PublicKey::from_bytes(public_key)
            .map_err(|e| IcnError::Identity(format!("Invalid public key: {}", e)))?;
        let message = self.hash().as_bytes().to_vec();
        if let Some(signature) = &self.signature {
            let signature = ed25519_dalek::Signature::from_bytes(signature)
                .map_err(|e| IcnError::Identity(format!("Invalid signature: {}", e)))?;
            Ok(public_key.verify(&message, &signature).is_ok())
        } else {
            Ok(false)
        }
    }

    pub fn add_zkp(&mut self, proof: Vec<u8>, public_inputs: Vec<Vec<u8>>) {
        self.zkp = Some(ZKProof {
            proof,
            public_inputs,
        });
    }

    pub fn verify_zkp<E: bellman::Engine, C: ZKCircuit<E>>(&self, circuit: C, verifying_key: &bellman::groth16::VerifyingKey<E>) -> IcnResult<bool> {
        if let Some(zkp) = &self.zkp {
            let proof = bellman::groth16::Proof::<E>::read(&zkp.proof[..])
                .map_err(|e| IcnError::ZKP(format!("Invalid ZKP: {}", e)))?;
            
            let public_inputs: Vec<E::Fr> = zkp.public_inputs.iter()
                .map(|input| E::Fr::from_str(&hex::encode(input)).unwrap())
                .collect();

            Ok(bellman::groth16::verify_proof(verifying_key, &proof, &public_inputs)
                .map_err(|e| IcnError::ZKP(format!("ZKP verification failed: {}", e)))?)
        } else {
            Ok(false)
        }
    }
}

/// Implementation of methods for Block.
impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            zkp_accumulator: None,
        };
        block.hash = block.hash();
        block
    }

    pub fn genesis() -> Self {
        Block::new(0, Vec::new(), "0".repeat(64))
    }

    pub fn add_zkp_accumulator(&mut self, accumulator: ZKAccumulator) {
        self.zkp_accumulator = Some(accumulator);
    }
}

/// Structure representing a ZK accumulator circuit.
pub struct ZKAccumulatorCircuit<E: bellman::Engine> {
    pub transactions: Vec<Transaction>,
    pub previous_accumulator: Option<E::Fr>,
}

/// Implementation of the ZKCircuit trait for ZKAccumulatorCircuit.
impl<E: bellman::Engine> ZKCircuit<E> for ZKAccumulatorCircuit<E> {
    fn synthesize<CS: ConstraintSystem<E>>(
        self,
        cs: &mut CS
    ) -> Result<(), SynthesisError> {
        // Implementation of the ZKP circuit for accumulating transactions
        // This is a placeholder and should be implemented based on the specific requirements
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use rand::SeedableRng;
    use rand_chacha::ChaChaRng;

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );
        let hash = tx.hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_block_hash() {
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );
        let block = Block::new(1, vec![tx], "previous_hash".to_string());
        let hash = block.hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
        assert_eq!(block.hash, hash);
    }

    #[test]
    fn test_transaction_sign_and_verify() {
        use ed25519_dalek::{Keypair, Signer};

        let mut rng = ChaChaRng::from_entropy();
        let keypair: Keypair = Keypair::generate(&mut rng);

        let mut tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );

        tx.sign(keypair.to_bytes().as_ref()).unwrap();
        assert!(tx.verify(keypair.public.as_bytes()).unwrap());

        // Test with wrong public key
        let wrong_keypair: Keypair = Keypair::generate(&mut rng);
        assert!(!tx.verify(wrong_keypair.public.as_bytes()).unwrap());
    }

    #[test]
    fn test_zkp_integration() {
        use bellman::groth16;
        use bls12_381::Bls12;

        // This is a dummy circuit for testing purposes
        struct DummyCircuit;
        impl ZKCircuit<Bls12> for DummyCircuit {
            fn synthesize<CS: ConstraintSystem<Bls12>>(
                self,
                _cs: &mut CS
            ) -> Result<(), SynthesisError> {
                Ok(())
            }
        }

        let mut rng = ChaChaRng::from_entropy();
        let params = groth16::generate_random_parameters::<Bls12, _, _>(DummyCircuit, &mut rng).unwrap();
        let pvk = groth16::prepare_verifying_key(&params.vk);

        let mut tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1234567890,
        );

        let proof = groth16::create_random_proof(DummyCircuit, &params, &mut rng).unwrap();
        let proof_vec = proof.write(&mut vec![]).unwrap();
        tx.add_zkp(proof_vec, vec![]);

        assert!(tx.verify_zkp(DummyCircuit, &pvk).unwrap());
    }
}
