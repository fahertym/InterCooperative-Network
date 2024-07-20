use serde::{Deserialize, Serialize};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use crate::types::CurrencyType; // Import CurrencyType from crate
use bincode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub gas_limit: u64,
    pub currency_type: CurrencyType, // Add this field
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64, gas_limit: u64, currency_type: CurrencyType) -> Self {
        Transaction {
            from,
            to,
            amount,
            gas_limit,
            currency_type, // Initialize the field
            signature: None,
            public_key: None,
        }
    }

    pub fn sign(&mut self, keypair: &Keypair) -> Result<(), String> {
        let message = self.to_bytes();
        let signature = keypair.sign(&message);
        self.signature = Some(signature.to_bytes().to_vec());
        self.public_key = Some(keypair.public.to_bytes().to_vec());
        Ok(())
    }

    pub fn verify(&self) -> Result<bool, String> {
        let public_key_bytes = self.public_key.as_ref().ok_or("No public key present")?;
        let signature_bytes = self.signature.as_ref().ok_or("No signature present")?;

        let public_key = PublicKey::from_bytes(public_key_bytes).map_err(|e| e.to_string())?;
        let signature = Signature::from_bytes(signature_bytes).map_err(|e| e.to_string())?;

        let message = self.to_bytes();
        Ok(public_key.verify(&message, &signature).is_ok())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.from.as_bytes());
        bytes.extend_from_slice(self.to.as_bytes());
        bytes.extend_from_slice(&self.amount.to_le_bytes());
        bytes.extend_from_slice(&self.gas_limit.to_le_bytes());
        // Serialize currency_type and append to bytes
        let currency_type_bytes = bincode::serialize(&self.currency_type).unwrap();
        bytes.extend_from_slice(&currency_type_bytes);
        bytes
    }

    pub fn to_string(&self) -> String {
        format!("{}{}{}{}", self.from, self.to, self.amount, self.gas_limit)
    }

    pub fn is_signed(&self) -> bool {
        self.signature.is_some() && self.public_key.is_some()
    }

    pub fn get_fee(&self) -> f64 {
        self.gas_limit as f64 * 0.0001
    }
}
