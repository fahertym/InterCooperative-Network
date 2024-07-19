use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use icn_core::error::{Error, Result};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetToken {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub last_transferred: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl AssetToken {
    pub fn new(name: String, description: String, owner: String, metadata: serde_json::Value) -> Self {
        let now = Utc::now();
        AssetToken {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            owner,
            created_at: now,
            last_transferred: now,
            metadata,
        }
    }

    pub fn transfer(&mut self, new_owner: String) {
        self.owner = new_owner;
        self.last_transferred = Utc::now();
    }
}

pub struct AssetRegistry {
    tokens: HashMap<String, AssetToken>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        AssetRegistry {
            tokens: HashMap::new(),
        }
    }

    pub fn create_token(&mut self, name: String, description: String, owner: String, metadata: serde_json::Value) -> AssetToken {
        let token = AssetToken::new(name, description, owner, metadata);
        self.tokens.insert(token.id.clone(), token.clone());
        token
    }

    pub fn transfer_token(&mut self, token_id: &str, new_owner: String) -> Result<()> {
        if let Some(token) = self.tokens.get_mut(token_id) {
            token.transfer(new_owner);
            Ok(())
        } else {
            Err(Error::BlockchainError("Token not found".to_string()))
        }
    }

    pub fn get_token(&self, token_id: &str) -> Option<&AssetToken> {
        self.tokens.get(token_id)
    }

    pub fn get_tokens_by_owner(&self, owner: &str) -> Vec<&AssetToken> {
        self.tokens.values().filter(|token| token.owner == owner).collect()
    }

    pub fn update_token_metadata(&mut self, token_id: &str, metadata: serde_json::Value) -> Result<()> {
        if let Some(token) = self.tokens.get_mut(token_id) {
            token.metadata = metadata;
            Ok(())
        } else {
            Err(Error::BlockchainError("Token not found".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_tokenization() {
        let mut registry = AssetRegistry::new();
        
        let metadata = serde_json::json!({
            "type": "Real Estate",
            "location": "123 Main St, Anytown, USA",
            "square_feet": 2000
        });
        
        let token = registry.create_token(
            "Main Street Property".to_string(),
            "A beautiful property on Main Street".to_string(),
            "Alice".to_string(),
            metadata
        );
        
        assert_eq!(token.owner, "Alice");
        
        registry.transfer_token(&token.id, "Bob".to_string()).unwrap();
        
        let updated_token = registry.get_token(&token.id).unwrap();
        assert_eq!(updated_token.owner, "Bob");
    }

    #[test]
    fn test_get_tokens_by_owner() {
        let mut registry = AssetRegistry::new();

        let metadata = serde_json::json!({});

        registry.create_token("Token1".to_string(), "Description1".to_string(), "Alice".to_string(), metadata.clone());
        registry.create_token("Token2".to_string(), "Description2".to_string(), "Alice".to_string(), metadata.clone());
        registry.create_token("Token3".to_string(), "Description3".to_string(), "Bob".to_string(), metadata.clone());

        let alice_tokens = registry.get_tokens_by_owner("Alice");
        assert_eq!(alice_tokens.len(), 2);

        let bob_tokens = registry.get_tokens_by_owner("Bob");
        assert_eq!(bob_tokens.len(), 1);
    }

    #[test]
    fn test_update_token_metadata() {
        let mut registry = AssetRegistry::new();

        let initial_metadata = serde_json::json!({
            "type": "Art",
            "artist": "Unknown"
        });

        let token = registry.create_token(
            "Mysterious Painting".to_string(),
            "An intriguing work of art".to_string(),
            "Alice".to_string(),
            initial_metadata
        );

        let new_metadata = serde_json::json!({
            "type": "Art",
            "artist": "Leonardo da Vinci",
            "year": 1503
        });

        registry.update_token_metadata(&token.id, new_metadata.clone()).unwrap();

        let updated_token = registry.get_token(&token.id).unwrap();
        assert_eq!(updated_token.metadata, new_metadata);
    }
}