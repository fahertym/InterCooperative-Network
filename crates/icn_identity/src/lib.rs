// File: crates/icn_identity/src/lib.rs

use icn_common::{IcnResult, IcnError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, warn, error};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecentralizedIdentity {
    pub id: String,
    pub public_key: PublicKey,
    pub attributes: HashMap<String, String>,
    pub reputation: f64,
}

pub struct IdentityManager {
    identities: Arc<RwLock<HashMap<String, DecentralizedIdentity>>>,
}

impl IdentityManager {
    pub fn new() -> Self {
        IdentityManager {
            identities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_identity(&self, attributes: HashMap<String, String>) -> IcnResult<(DecentralizedIdentity, Keypair)> {
        let mut csprng = OsRng{};
        let keypair = Keypair::generate(&mut csprng);
        let public_key = keypair.public;

        let id = format!("did:icn:{}", hex::encode(public_key.as_bytes()));

        let identity = DecentralizedIdentity {
            id: id.clone(),
            public_key,
            attributes,
            reputation: 1.0, // Initial reputation
        };

        let mut identities = self.identities.write().map_err(|_| IcnError::Identity("Failed to lock identities".into()))?;
        identities.insert(id.clone(), identity.clone());

        info!("Created new identity: {}", id);
        Ok((identity, keypair))
    }

    pub fn get_identity(&self, id: &str) -> IcnResult<DecentralizedIdentity> {
        let identities = self.identities.read().map_err(|_| IcnError::Identity("Failed to lock identities".into()))?;
        identities.get(id)
            .cloned()
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))
    }

    pub fn update_attributes(&self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        let mut identities = self.identities.write().map_err(|_| IcnError::Identity("Failed to lock identities".into()))?;
        let identity = identities.get_mut(id).ok_or_else(|| IcnError::Identity("Identity not found".into()))?;
        identity.attributes.extend(attributes);
        Ok(())
    }

    pub fn update_reputation(&self, id: &str, reputation_change: f64) -> IcnResult<()> {
        let mut identities = self.identities.write().map_err(|_| IcnError::Identity("Failed to lock identities".into()))?;
        let identity = identities.get_mut(id).ok_or_else(|| IcnError::Identity("Identity not found".into()))?;
        identity.reputation += reputation_change;
        identity.reputation = identity.reputation.max(0.0).min(100.0); // Clamp reputation between 0 and 100
        Ok(())
    }

    pub fn verify_signature(&self, id: &str, message: &[u8], signature: &Signature) -> IcnResult<bool> {
        let identities = self.identities.read().map_err(|_| IcnError::Identity("Failed to lock identities".into()))?;
        let identity = identities.get(id).ok_or_else(|| IcnError::Identity("Identity not found".into()))?;
        Ok(identity.public_key.verify(message, signature).is_ok())
    }

    pub fn list_identities(&self) -> IcnResult<Vec<DecentralizedIdentity>> {
        let identities = self.identities.read().map_err(|_| IcnError::Identity("Failed to lock identities".into()))?;
        Ok(identities.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_identity() {
        let identity_manager = IdentityManager::new();
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        attributes.insert("email".to_string(), "alice@example.com".to_string());

        let (identity, _keypair) = identity_manager.create_identity(attributes).unwrap();
        let retrieved_identity = identity_manager.get_identity(&identity.id).unwrap();

        assert_eq!(identity.id, retrieved_identity.id);
        assert_eq!(identity.attributes, retrieved_identity.attributes);
        assert_eq!(identity.reputation, retrieved_identity.reputation);
    }

    #[test]
    fn test_update_attributes() {
        let identity_manager = IdentityManager::new();
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Bob".to_string());

        let (identity, _keypair) = identity_manager.create_identity(attributes).unwrap();

        let mut new_attributes = HashMap::new();
        new_attributes.insert("age".to_string(), "30".to_string());

        identity_manager.update_attributes(&identity.id, new_attributes).unwrap();

        let updated_identity = identity_manager.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.attributes.get("name"), Some(&"Bob".to_string()));
        assert_eq!(updated_identity.attributes.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_update_reputation() {
        let identity_manager = IdentityManager::new();
        let (identity, _keypair) = identity_manager.create_identity(HashMap::new()).unwrap();

        identity_manager.update_reputation(&identity.id, 0.5).unwrap();
        let updated_identity = identity_manager.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.reputation, 1.5);

        // Test clamping
        identity_manager.update_reputation(&identity.id, 1000.0).unwrap();
        let clamped_identity = identity_manager.get_identity(&identity.id).unwrap();
        assert_eq!(clamped_identity.reputation, 100.0);
    }

    #[test]
    fn test_verify_signature() {
        let identity_manager = IdentityManager::new();
        let (identity, keypair) = identity_manager.create_identity(HashMap::new()).unwrap();

        let message = b"Hello, world!";
        let signature = keypair.sign(message);

        assert!(identity_manager.verify_signature(&identity.id, message, &signature).unwrap());

        // Test with wrong message
        let wrong_message = b"Wrong message";
        assert!(!identity_manager.verify_signature(&identity.id, wrong_message, &signature).unwrap());
    }

    #[test]
    fn test_list_identities() {
        let identity_manager = IdentityManager::new();
        identity_manager.create_identity(HashMap::new()).unwrap();
        identity_manager.create_identity(HashMap::new()).unwrap();

        let identities = identity_manager.list_identities().unwrap();
        assert_eq!(identities.len(), 2);
    }
}