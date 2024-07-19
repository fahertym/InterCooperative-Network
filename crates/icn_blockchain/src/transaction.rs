use serde::{Deserialize, Serialize};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use icn_currency::CurrencyType;
use rand::rngs::OsRng;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub gas_limit: u64,
    pub smart_contract_id: Option<String>,
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType, gas_limit: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            gas_limit,
            smart_contract_id: None,
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
        bytes.extend_from_slice(&serde_json::to_vec(&self.currency_type).unwrap());
        if let Some(contract_id) = &self.smart_contract_id {
            bytes.extend_from_slice(contract_id.as_bytes());
        }
        bytes
    }

    pub fn to_string(&self) -> String {
        format!("{}{}{}{:?}{}", self.from, self.to, self.amount, self.currency_type, self.gas_limit)
    }

    pub fn with_smart_contract(&mut self, smart_contract_id: String) -> &mut Self {
        self.smart_contract_id = Some(smart_contract_id);
        self
    }

    pub fn is_signed(&self) -> bool {
        self.signature.is_some() && self.public_key.is_some()
    }

    pub fn get_fee(&self) -> f64 {
        // This is a simplified fee calculation. In a real system, this would be more complex.
        self.gas_limit as f64 * 0.0001
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transaction() {
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        assert_eq!(tx.from, "Alice");
        assert_eq!(tx.to, "Bob");
        assert_eq!(tx.amount, 100.0);
        assert_eq!(tx.currency_type, CurrencyType::BasicNeeds);
        assert_eq!(tx.gas_limit, 1000);
    }

    #[test]
    fn test_sign_and_verify_transaction() {
        let mut tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        assert!(tx.sign(&keypair).is_ok());
        assert!(tx.is_signed());
        assert!(tx.verify().unwrap());
    }

    #[test]
    fn test_with_smart_contract() {
        let mut tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        tx.with_smart_contract("contract123".to_string());
        assert_eq!(tx.smart_contract_id, Some("contract123".to_string()));
    }

    #[test]
    fn test_get_fee() {
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        assert_eq!(tx.get_fee(), 0.1);
    }
}