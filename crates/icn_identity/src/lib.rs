mod did;
mod identity_manager;

pub use did::DecentralizedIdentity;
pub use identity_manager::IdentityManager;

use icn_common::{IcnError, IcnResult};
use std::collections::HashMap;
use ed25519_dalek::Signature;

pub struct IdentityService {
    manager: IdentityManager,
}

impl IdentityService {
    pub fn new() -> Self {
        IdentityService {
            manager: IdentityManager::new(),
        }
    }

    pub fn create_identity(&mut self, attributes: HashMap<String, String>) -> IcnResult<DecentralizedIdentity> {
        self.manager.create_identity(attributes)
    }

    pub fn get_identity(&self, id: &str) -> IcnResult<&DecentralizedIdentity> {
        self.manager.get_identity(id)
    }

    pub fn update_attributes(&mut self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        self.manager.update_attributes(id, attributes)
    }

    pub fn update_reputation(&mut self, id: &str, change: f64) -> IcnResult<()> {
        self.manager.update_reputation(id, change)
    }

    pub fn verify_signature(&self, id: &str, message: &[u8], signature: &Signature) -> IcnResult<bool> {
        self.manager.verify_signature(id, message, signature)
    }

    pub fn list_identities(&self) -> Vec<&DecentralizedIdentity> {
        self.manager.list_identities()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Keypair, Signer};

    #[test]
    fn test_identity_service() {
        let mut service = IdentityService::new();

        // Create a new identity
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        let identity = service.create_identity(attributes).unwrap();

        // Verify the identity exists
        assert!(service.get_identity(&identity.id).is_ok());

        // Update attributes
        let mut new_attributes = HashMap::new();
        new_attributes.insert("age".to_string(), "30".to_string());
        assert!(service.update_attributes(&identity.id, new_attributes).is_ok());

        // Update reputation
        assert!(service.update_reputation(&identity.id, 0.5).is_ok());
        let updated_identity = service.get_identity(&identity.id).unwrap();
        assert_eq!(updated_identity.reputation, 1.5);

        // Test signature verification
        let message = b"Hello, World!";
        let mut csprng = rand::rngs::OsRng{};
        let keypair = Keypair::generate(&mut csprng);
        let signature = keypair.sign(message);

        // This should fail because we're using a different keypair
        assert!(!service.verify_signature(&identity.id, message, &signature).unwrap());

        // List identities
        let identities = service.list_identities();
        assert_eq!(identities.len(), 1);
        assert_eq!(identities[0].id, identity.id);
    }
}