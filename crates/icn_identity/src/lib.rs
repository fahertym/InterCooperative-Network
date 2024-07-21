use icn_common::{IcnResult, IcnError};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecentralizedIdentity {
    pub id: String,
    #[serde(with = "public_key_serde")]
    pub public_key: PublicKey,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub reputation: f64,
    pub attributes: HashMap<String, String>,
    pub status: IdentityStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum IdentityStatus {
    Active,
    Suspended,
    Revoked,
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
    pub fn new(attributes: HashMap<String, String>) -> IcnResult<(Self, Keypair)> {
        let mut csprng = OsRng {};
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let public_key = keypair.public;
        let id = format!("did:icn:{}", hex::encode(public_key.to_bytes()));
        let now = Utc::now();

        Ok((
            Self {
                id,
                public_key,
                created_at: now,
                updated_at: now,
                reputation: 1.0,
                attributes,
                status: IdentityStatus::Active,
            },
            keypair,
        ))
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
        let (identity, _) = DecentralizedIdentity::new(attributes)?;
        self.identities.insert(identity.id.clone(), identity.clone());
        Ok(identity)
    }

    pub fn get_identity(&self, id: &str) -> IcnResult<&DecentralizedIdentity> {
        self.identities.get(id).ok_or(IcnError::Identity("Identity not found".to_string()))
    }

    pub fn update_identity(&mut self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        let identity = self.identities.get_mut(id).ok_or(IcnError::Identity("Identity not found".to_string()))?;
        identity.attributes.extend(attributes);
        identity.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_reputation(&mut self, id: &str, change: f64) -> IcnResult<()> {
        let identity = self.identities.get_mut(id).ok_or(IcnError::Identity("Identity not found".to_string()))?;
        identity.reputation += change;
        identity.updated_at = Utc::now();
        Ok(())
    }

    pub fn suspend_identity(&mut self, id: &str) -> IcnResult<()> {
        let identity = self.identities.get_mut(id).ok_or(IcnError::Identity("Identity not found".to_string()))?;
        identity.status = IdentityStatus::Suspended;
        identity.updated_at = Utc::now();
        Ok(())
    }

    pub fn revoke_identity(&mut self, id: &str) -> IcnResult<()> {
        let identity = self.identities.get_mut(id).ok_or(IcnError::Identity("Identity not found".to_string()))?;
        identity.status = IdentityStatus::Revoked;
        identity.updated_at = Utc::now();
        Ok(())
    }

    pub fn verify_signature(&self, id: &str, message: &[u8], signature: &Signature) -> IcnResult<bool> {
        let identity = self.identities.get(id).ok_or(IcnError::Identity("Identity not found".to_string()))?;
        Ok(identity.verify_signature(message, signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

    #[test]
    fn test_identity_creation_and_verification() {
        let mut manager = IdentityManager::new();
        
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        
        let identity = manager.create_identity(attributes).unwrap();
        
        assert!(manager.get_identity(&identity.id).is_ok());
        
        let message = b"Hello, World!";
        let mut csprng = OsRng {};
        let keypair = Keypair::generate(&mut csprng);
        let signature = keypair.sign(message);
        
        // This should fail because we're using a different keypair
        assert!(!manager.verify_signature(&identity.id, message, &signature).unwrap());
        
        // Create a valid signature
        let (_, keypair) = DecentralizedIdentity::new(HashMap::new()).unwrap();
        let valid_signature = keypair.sign(message);
        
        // This should succeed
        assert!(manager.verify_signature(&identity.id, message, &valid_signature).unwrap());
    }

    #[test]
    fn test_identity_management() {
        let mut manager = IdentityManager::new();
        
        // Create an identity
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        let identity = manager.create_identity(attributes).unwrap();
        
        // Update identity
        let mut new_attributes = HashMap::new();
        new_attributes.insert("email".to_string(), "alice@example.com".to_string());
        assert!(manager.update_identity(&identity.id, new_attributes).is_ok());
        
        // Check updated identity
        let updated_identity = manager.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.attributes.get("email"), Some(&"alice@example.com".to_string()));
        
        // Update reputation
        assert!(manager.update_reputation(&identity.id, 0.5).is_ok());
        let updated_identity = manager.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.reputation, 1.5);
        
        // Suspend identity
        assert!(manager.suspend_identity(&identity.id).is_ok());
        let suspended_identity = manager.get_identity(&identity.id).unwrap();
        assert_eq!(suspended_identity.status, IdentityStatus::Suspended);
        
        // Revoke identity
        assert!(manager.revoke_identity(&identity.id).is_ok());
        let revoked_identity = manager.get_identity(&identity.id).unwrap();
        assert_eq!(revoked_identity.status, IdentityStatus::Revoked);
    }
}