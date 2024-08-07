// File: crates/icn_identity/src/lib.rs

use icn_common::{IcnResult, IcnError};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecentralizedIdentity {
    pub id: String,
    pub public_key: PublicKey,
    pub created_at: DateTime<Utc>,
    pub reputation: f64,
    pub attributes: HashMap<String, String>,
    pub revoked: bool,
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
                revoked: false,
            },
            keypair,
        )
    }

    pub fn verify_signature(&self, message: &[u8], signature: &Signature) -> bool {
        self.public_key.verify(message, signature).is_ok()
    }
}

pub struct IdentityService {
    identities: HashMap<String, DecentralizedIdentity>,
}

impl IdentityService {
    pub fn new() -> Self {
        IdentityService {
            identities: HashMap::new(),
        }
    }

    pub fn create_identity(&mut self, attributes: HashMap<String, String>) -> IcnResult<DecentralizedIdentity> {
        let (identity, _) = DecentralizedIdentity::new(attributes);

        if self.identities.contains_key(&identity.id) {
            return Err(IcnError::Identity("Identity already exists".into()));
        }

        let id = identity.id.clone();
        self.identities.insert(id, identity.clone());
        Ok(identity)
    }

    pub fn get_identity(&self, id: &str) -> IcnResult<&DecentralizedIdentity> {
        self.identities.get(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))
    }

    pub fn update_attributes(&mut self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        let identity = self.identities.get_mut(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))?;

        identity.attributes.extend(attributes);
        Ok(())
    }

    pub fn update_reputation(&mut self, id: &str, change: f64) -> IcnResult<()> {
        let identity = self.identities.get_mut(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))?;
        
        identity.reputation += change;
        
        // Ensure reputation stays within a reasonable range (e.g., 0 to 100)
        identity.reputation = identity.reputation.max(0.0).min(100.0);
        
        Ok(())
    }

    pub fn verify_signature(&self, id: &str, message: &[u8], signature: &Signature) -> IcnResult<bool> {
        let identity = self.get_identity(id)?;
        Ok(identity.verify_signature(message, signature))
    }

    pub fn list_identities(&self) -> Vec<&DecentralizedIdentity> {
        self.identities.values().collect()
    }

    pub fn remove_identity(&mut self, id: &str) -> IcnResult<()> {
        self.identities.remove(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))?;
        Ok(())
    }

    pub fn get_reputation(&self, id: &str) -> IcnResult<f64> {
        let identity = self.get_identity(id)?;
        Ok(identity.reputation)
    }

    pub fn get_attribute(&self, id: &str, attribute_key: &str) -> IcnResult<Option<String>> {
        let identity = self.get_identity(id)?;
        Ok(identity.attributes.get(attribute_key).cloned())
    }

    pub fn set_attribute(&mut self, id: &str, key: String, value: String) -> IcnResult<()> {
        let identity = self.identities.get_mut(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))?;
        identity.attributes.insert(key, value);
        Ok(())
    }

    pub fn revoke_identity(&mut self, id: &str) -> IcnResult<()> {
        let identity = self.identities.get_mut(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))?;
        
        if identity.revoked {
            return Err(IcnError::Identity("Identity is already revoked".into()));
        }

        identity.revoked = true;
        self.broadcast_revocation(id)?;
        Ok(())
    }

    pub fn update_identity(&mut self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        let identity = self.identities.get_mut(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))?;
        
        if identity.revoked {
            return Err(IcnError::Identity("Cannot update a revoked identity".into()));
        }

        identity.attributes.extend(attributes);
        Ok(())
    }

    fn broadcast_revocation(&self, id: &str) -> IcnResult<()> {
        // This is a placeholder for the actual network broadcast implementation
        println!("Broadcasting revocation of identity: {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_creation_and_retrieval() {
        let mut service = IdentityService::new();
        
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        attributes.insert("email".to_string(), "alice@example.com".to_string());
        
        let identity = service.create_identity(attributes.clone()).unwrap();
        assert!(identity.id.starts_with("did:icn:"));
        
        let retrieved_identity = service.get_identity(&identity.id).unwrap();
        assert_eq!(retrieved_identity.attributes, attributes);
    }

    #[test]
    fn test_update_attributes() {
        let mut service = IdentityService::new();
        
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        
        let identity = service.create_identity(attributes).unwrap();
        
        let mut new_attributes = HashMap::new();
        new_attributes.insert("email".to_string(), "alice@example.com".to_string());
        
        service.update_attributes(&identity.id, new_attributes).unwrap();
        
        let updated_identity = service.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.attributes.get("name"), Some(&"Alice".to_string()));
        assert_eq!(updated_identity.attributes.get("email"), Some(&"alice@example.com".to_string()));
    }

    #[test]
    fn test_update_reputation() {
        let mut service = IdentityService::new();
        
        let attributes = HashMap::new();
        let identity = service.create_identity(attributes).unwrap();
        
        service.update_reputation(&identity.id, 0.5).unwrap();
        let updated_identity = service.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.reputation, 1.5);

        service.update_reputation(&identity.id, -0.2).unwrap();
        let updated_identity = service.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.reputation, 1.3);

        // Testing reputation range enforcement
        service.update_reputation(&identity.id, 99.0).unwrap();
        let updated_identity = service.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.reputation, 100.0);

        service.update_reputation(&identity.id, -200.0).unwrap();
        let updated_identity = service.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.reputation, 0.0);
    }

    #[test]
    fn test_signature_verification() {
        let mut service = IdentityService::new();
        
        let attributes = HashMap::new();
        let (identity, keypair) = DecentralizedIdentity::new(attributes);
        service.identities.insert(identity.id.clone(), identity);
        
        let message = b"Hello, World!";
        let signature = keypair.sign(message);
        
        assert!(service.verify_signature(&identity.id, message, &signature).unwrap());
    }

    #[test]
    fn test_list_identities() {
        let mut service = IdentityService::new();
        
        let attributes1 = HashMap::new();
        let attributes2 = HashMap::new();
        
        service.create_identity(attributes1).unwrap();
        service.create_identity(attributes2).unwrap();
        
        let identities = service.list_identities();
        assert_eq!(identities.len(), 2);
    }

    #[test]
    fn test_remove_identity() {
        let mut service = IdentityService::new();
        
        let attributes = HashMap::new();
        let identity = service.create_identity(attributes).unwrap();
        
        assert!(service.remove_identity(&identity.id).is_ok());
        assert!(service.get_identity(&identity.id).is_err());
    }

    #[test]
    fn test_get_and_set_attribute() {
        let mut service = IdentityService::new();
        
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        
        let identity = service.create_identity(attributes).unwrap();
        
        assert_eq!(service.get_attribute(&identity.id, "name").unwrap(), Some("Alice".to_string()));
        assert_eq!(service.get_attribute(&identity.id, "email").unwrap(), None);
        
        service.set_attribute(&identity.id, "email".to_string(), "alice@example.com".to_string()).unwrap();
        assert_eq!(service.get_attribute(&identity.id, "email").unwrap(), Some("alice@example.com".to_string()));
    }

    #[test]
    fn test_revoke_identity() {
        let mut service = IdentityService::new();
        
        let attributes = HashMap::new();
        let identity = service.create_identity(attributes).unwrap();
        
        assert!(!identity.revoked);
        
        service.revoke_identity(&identity.id).unwrap();
        
        let revoked_identity = service.get_identity(&identity.id).unwrap();
        assert!(revoked_identity.revoked);
        
        // Trying to revoke an already revoked identity should fail
        assert!(service.revoke_identity(&identity.id).is_err());
    }

    #[test]
    fn test_update_identity() {
        let mut service = IdentityService::new();
        
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        
        let identity = service.create_identity(attributes).unwrap();
        
        let mut new_attributes = HashMap::new();
        new_attributes.insert("email".to_string(), "alice@example.com".to_string());
        
        service.update_identity(&identity.id, new_attributes).unwrap();
        
        let updated_identity = service.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.attributes.get("name"), Some(&"Alice".to_string()));
        assert_eq!(updated_identity.attributes.get("email"), Some(&"alice@example.com".to_string()));
        
        // Revoking the identity
        service.revoke_identity(&identity.id).unwrap();
        
        // Trying to update a revoked identity should fail
        let another_update = HashMap::new();
        assert!(service.update_identity(&identity.id, another_update).is_err());
    }
}