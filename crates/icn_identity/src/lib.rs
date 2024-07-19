use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, Signature, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecentralizedIdentity {
    pub id: String,
    #[serde(with = "public_key_serde")]
    pub public_key: PublicKey,
    pub created_at: DateTime<Utc>,
    pub reputation: f64,
    pub attributes: HashMap<String, String>,
}

mod public_key_serde {
    use ed25519_dalek::PublicKey;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(public_key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = public_key.to_bytes();
        bytes.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        PublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

impl DecentralizedIdentity {
    pub fn new(attributes: HashMap<String, String>) -> (Self, Keypair) {
        let mut csprng = OsRng {};
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let public_key = keypair.public;
        let id = format!("did:icn:{}", hex::encode(public_key.to_bytes()));

        (
            Self {
                id,
                public_key,
                created_at: Utc::now(),
                reputation: 1.0,
                attributes,
            },
            keypair,
        )
    }

    pub fn verify_signature(&self, message: &[u8], signature: &Signature) -> bool {
        self.public_key.verify(message, signature).is_ok()
    }
}

pub struct DidManager {
    dids: HashMap<String, DecentralizedIdentity>,
}

impl DidManager {
    pub fn new() -> Self {
        DidManager {
            dids: HashMap::new(),
        }
    }

    pub fn add_did(&mut self, did: DecentralizedIdentity) {
        self.dids.insert(did.id.clone(), did);
    }

    pub fn get_did(&self, id: &str) -> Option<&DecentralizedIdentity> {
        self.dids.get(id)
    }

    pub fn verify_signature(
        &self,
        did_id: &str,
        message: &[u8],
        signature: &Signature,
    ) -> Result<bool, String> {
        if let Some(did) = self.get_did(did_id) {
            Ok(did.verify_signature(message, signature))
        } else {
            Err(format!("DID not found: {}", did_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

    #[test]
    fn test_add_and_get_did() {
        let mut manager = DidManager::new();
        let (did, _keypair) = DecentralizedIdentity::new(HashMap::new());
        let did_id = did.id.clone();
        manager.add_did(did);

        assert!(manager.get_did(&did_id).is_some());
    }

    #[test]
    fn test_verify_signature() {
        let mut manager = DidManager::new();
        let (did, keypair) = DecentralizedIdentity::new(HashMap::new());
        let did_id = did.id.clone();
        manager.add_did(did);

        let message = b"test message";
        let signature = keypair.sign(message);

        assert!(manager.verify_signature(&did_id, message, &signature).unwrap());
    }

    #[test]
    fn test_verify_signature_invalid() {
        let mut manager = DidManager::new();
        let (did, keypair) = DecentralizedIdentity::new(HashMap::new());
        let did_id = did.id.clone();
        manager.add_did(did);

        let message = b"test message";
        let invalid_message = b"invalid message";
        let signature = keypair.sign(message);

        assert!(!manager.verify_signature(&did_id, invalid_message, &signature).unwrap());
    }
}
