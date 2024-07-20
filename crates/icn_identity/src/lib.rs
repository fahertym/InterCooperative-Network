use icn_types::{IcnResult, IcnError};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
// In other crates like icn_core, icn_identity, etc.
use icn_common::{CommonError, CommonResult};


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
    use super::*;
    use serde::{Deserializer, Serializer};

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

pub struct IdentityManager {
    identities: HashMap<String, DecentralizedIdentity>,
}

impl IdentityManager {
    pub fn new() -> Self {
        IdentityManager {
            identities: HashMap::new(),
        }
    }

    pub fn create_identity(&mut self, attributes: HashMap<String, String>) -> IcnResult<DecentralizedIdentity> {
        let (identity, _) = DecentralizedIdentity::new(attributes);
        self.identities.insert(identity.id.clone(), identity.clone());
        Ok(identity)
    }

    pub fn get_identity(&self, id: &str) -> Option<&DecentralizedIdentity> {
        self.identities.get(id)
    }

    pub fn update_reputation(&mut self, id: &str, change: f64) -> IcnResult<()> {
        let identity = self.identities.get_mut(id).ok_or(IcnError::Identity("Identity not found".to_string()))?;
        identity.reputation += change;
        Ok(())
    }

    pub fn verify_signature(&self, id: &str, message: &[u8], signature: &Signature) -> IcnResult<bool> {
        let identity = self.identities.get(id).ok_or(IcnError::Identity("Identity not found".to_string()))?;
        Ok(identity.verify_signature(message, signature))
    }

    pub fn update_attributes(&mut self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        let identity = self.identities.get_mut(id).ok_or(IcnError::Identity("Identity not found".to_string()))?;
        identity.attributes.extend(attributes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_creation_and_verification() {
        let mut manager = IdentityManager::new();
        
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        
        let identity = manager.create_identity(attributes).unwrap();
        
        assert!(manager.get_identity(&identity.id).is_some());
        
        let message = b"Hello, World!";
        let mut csprng = OsRng {};
        let keypair = Keypair::generate(&mut csprng);
        let signature = keypair.sign(message);
        
        // This should fail because we're using a different keypair
        assert!(!manager.verify_signature(&identity.id, message, &signature).unwrap());
        
        /// Create a valid signature
        let (_, keypair) = DecentralizedIdentity::new(HashMap::new());
        let valid_signature = keypair.sign(message);
        
        // This should succeed
        assert!(manager.verify_signature(&identity.id, message, &valid_signature).unwrap());
    }

    #[test]
    fn test_reputation_update() {
        let mut manager = IdentityManager::new();
        let identity = manager.create_identity(HashMap::new()).unwrap();
        
        assert_eq!(identity.reputation, 1.0);
        
        manager.update_reputation(&identity.id, 0.5).unwrap();
        let updated_identity = manager.get_identity(&identity.id).unwrap();
        
        assert_eq!(updated_identity.reputation, 1.5);
    }

    #[test]
    fn test_attribute_update() {
        let mut manager = IdentityManager::new();
        let identity = manager.create_identity(HashMap::new()).unwrap();
        
        let mut new_attributes = HashMap::new();
        new_attributes.insert("role".to_string(), "developer".to_string());
        
        manager.update_attributes(&identity.id, new_attributes).unwrap();
        let updated_identity = manager.get_identity(&identity.id).unwrap();
        
        assert_eq!(updated_identity.attributes.get("role"), Some(&"developer".to_string()));
    }

    #[test]
    fn test_identity_not_found() {
        let mut manager = IdentityManager::new();
        
        assert!(manager.update_reputation("non_existent_id", 0.5).is_err());
        assert!(manager.verify_signature("non_existent_id", b"test", &Signature::new([0; 64])).is_err());
        assert!(manager.update_attributes("non_existent_id", HashMap::new()).is_err());
    }
}