// File: crates/icn_identity/src/lib.rs

use icn_common::{IcnError, IcnResult};
use std::collections::HashMap;
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;

#[derive(Clone)]
pub struct DecentralizedIdentity {
    pub id: String,
    pub public_key: PublicKey,
    pub attributes: HashMap<String, String>,
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
        let mut csprng = OsRng {};
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let public_key = keypair.public;

        let id = format!("did:icn:{}", hex::encode(public_key.as_bytes()));

        let identity = DecentralizedIdentity {
            id: id.clone(),
            public_key,
            attributes,
        };

        self.identities.insert(id.clone(), identity.clone());

        Ok(identity)
    }

    pub fn get_identity(&self, id: &str) -> IcnResult<&DecentralizedIdentity> {
        self.identities
            .get(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))
    }

    pub fn update_attributes(&mut self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        let identity = self
            .identities
            .get_mut(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))?;

        identity.attributes.extend(attributes);
        Ok(())
    }

    pub fn verify_signature(&self, id: &str, message: &[u8], signature: &Signature) -> IcnResult<bool> {
        let identity = self.get_identity(id)?;
        Ok(identity.public_key.verify(message, signature).is_ok())
    }

    pub fn list_identities(&self) -> Vec<&DecentralizedIdentity> {
        self.identities.values().collect()
    }
}

impl DecentralizedIdentity {
    pub fn sign(&self, keypair: &Keypair, message: &[u8]) -> Signature {
        keypair.sign(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;
    use rand::rngs::OsRng;

    #[test]
    fn test_create_identity() {
        let mut manager = IdentityManager::new();
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());

        let identity = manager.create_identity(attributes).unwrap();
        assert!(identity.id.starts_with("did:icn:"));
        assert_eq!(identity.attributes.get("name"), Some(&"Alice".to_string()));
    }

    #[test]
    fn test_get_identity() {
        let mut manager = IdentityManager::new();
        let attributes = HashMap::new();
        let identity = manager.create_identity(attributes).unwrap();

        let retrieved_identity = manager.get_identity(&identity.id).unwrap();
        assert_eq!(retrieved_identity.id, identity.id);
    }

    #[test]
    fn test_update_attributes() {
        let mut manager = IdentityManager::new();
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());

        let identity = manager.create_identity(attributes).unwrap();

        let mut new_attributes = HashMap::new();
        new_attributes.insert("age".to_string(), "30".to_string());

        manager.update_attributes(&identity.id, new_attributes).unwrap();

        let updated_identity = manager.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.attributes.get("name"), Some(&"Alice".to_string()));
        assert_eq!(updated_identity.attributes.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_verify_signature() {
        let mut manager = IdentityManager::new();
        let attributes = HashMap::new();

        let mut csprng = OsRng {};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let identity = manager.create_identity(attributes).unwrap();

        let message = b"Hello, world!";
        let signature = identity.sign(&keypair, message);

        assert!(manager.verify_signature(&identity.id, message, &signature).unwrap());

        // Test with incorrect message
        let wrong_message = b"Wrong message";
        assert!(!manager.verify_signature(&identity.id, wrong_message, &signature).unwrap());
    }

    #[test]
    fn test_list_identities() {
        let mut manager = IdentityManager::new();
        let attributes = HashMap::new();

        manager.create_identity(attributes.clone()).unwrap();
        manager.create_identity(attributes.clone()).unwrap();

        let identities = manager.list_identities();
        assert_eq!(identities.len(), 2);
    }
}
