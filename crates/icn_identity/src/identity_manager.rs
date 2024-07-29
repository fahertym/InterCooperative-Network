use crate::DecentralizedIdentity;
use ed25519_dalek::Signature;
use icn_common::{IcnError, IcnResult};
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

    pub fn create_identity(&mut self, attributes: HashMap<String, String>) -> IcnResult<DecentralizedIdentity> {
        let (identity, _) = DecentralizedIdentity::new(attributes);
        self.identities.insert(identity.id.clone(), identity.clone());
        Ok(identity)
    }

    pub fn get_identity(&self, id: &str) -> IcnResult<&DecentralizedIdentity> {
        self.identities
            .get(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))
    }

    pub fn update_attributes(&mut self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        let identity = self.identities
            .get_mut(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))?;

        identity.attributes.extend(attributes);
        Ok(())
    }

    pub fn update_reputation(&mut self, id: &str, change: f64) -> IcnResult<()> {
        let identity = self.identities
            .get_mut(id)
            .ok_or_else(|| IcnError::Identity("Identity not found".into()))?;
        identity.reputation += change;
        Ok(())
    }

    pub fn verify_signature(&self, id: &str, message: &[u8], signature: &Signature) -> IcnResult<bool> {
        let identity = self.get_identity(id)?;
        Ok(identity.verify_signature(message, signature))
    }

    pub fn list_identities(&self) -> Vec<&DecentralizedIdentity> {
        self.identities.values().collect()
    }
}

impl Default for IdentityManager {
    fn default() -> Self {
        Self::new()
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
        assert!(manager.get_identity(&identity.id).is_ok());
        
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
