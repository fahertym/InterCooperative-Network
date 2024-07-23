use crate::DecentralizedIdentity;
use ed25519_dalek::Signature;
use icn_core::error::{Error, Result};
use std::collections::HashMap;

pub struct IdentityManager {
    identities: HashMap<String, DecentralizedIdentity>,
}

impl IdentityManager {
    pub fn new() -> Self {
        IdentityManager {
            identities: HashMap::new(),
        }
    }

    pub fn create_identity(&mut self, attributes: HashMap<String, String>) -> Result<DecentralizedIdentity> {
        let (identity, _) = DecentralizedIdentity::new(attributes);
        self.identities.insert(identity.id.clone(), identity.clone());
        Ok(identity)
    }

    pub fn get_identity(&self, id: &str) -> Option<&DecentralizedIdentity> {
        self.identities.get(id)
    }

    pub fn update_reputation(&mut self, id: &str, change: f64) -> Result<()> {
        let identity = self.identities.get_mut(id).ok_or(Error::IdentityError("Identity not found".to_string()))?;
        identity.reputation += change;
        Ok(())
    }

    pub fn verify_signature(&self, id: &str, message: &[u8], signature: &Signature) -> Result<bool> {
        let identity = self.identities.get(id).ok_or(Error::IdentityError("Identity not found".to_string()))?;
        Ok(identity.verify_signature(message, signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Keypair, Signer};

    #[test]
    fn test_identity_management() {
        let mut manager = IdentityManager::new();
        
        // Create a new identity
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        let identity = manager.create_identity(attributes).unwrap();
        
        // Verify the identity exists
        assert!(manager.get_identity(&identity.id).is_some());
        
        // Update reputation
        assert!(manager.update_reputation(&identity.id, 0.5).is_ok());
        let updated_identity = manager.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.reputation, 1.5);
        
        // Test signature verification
        let message = b"Hello, World!";
        let mut csprng = rand::rngs::OsRng {};
        let keypair = Keypair::generate(&mut csprng);
        let signature = keypair.sign(message);
        
        // This should fail because we're using a different keypair
        assert!(!manager.verify_signature(&identity.id, message, &signature).unwrap());
    }
}
