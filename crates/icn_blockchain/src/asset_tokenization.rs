use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug)] // Add Debug derive
pub struct AssetRegistry {
    tokens: HashMap<String, AssetToken>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        AssetRegistry {
            tokens: HashMap::new(),
        }
    }

    pub fn create_token(&mut self, name: String, description: String, owner: String, metadata: serde_json::Value) -> Result<String, String> {
        let token = AssetToken::new(name, description, owner, metadata);
        let token_id = token.id.clone();
        self.tokens.insert(token_id.clone(), token);
        Ok(token_id)
    }

    pub fn transfer_token(&mut self, token_id: &str, new_owner: String) -> Result<(), String> {
        self.tokens.get_mut(token_id)
            .ok_or_else(|| "Token not found".to_string())
            .map(|token| token.transfer(new_owner))
    }

    pub fn get_token(&self, token_id: &str) -> Option<&AssetToken> {
        self.tokens.get(token_id)
    }

    pub fn list_tokens(&self) -> Vec<&AssetToken> {
        self.tokens.values().collect()
    }

    pub fn token_exists(&self, token_id: &str) -> bool {
        self.tokens.contains_key(token_id)
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
        
        let token_id = registry.create_token(
            "Main Street Property".to_string(),
            "A beautiful property on Main Street".to_string(),
            "Alice".to_string(),
            metadata
        ).unwrap();
        
        let token = registry.get_token(&token_id).unwrap();
        assert_eq!(token.owner, "Alice");
        
        registry.transfer_token(&token_id, "Bob".to_string()).unwrap();
        
        let updated_token = registry.get_token(&token_id).unwrap();
        assert_eq!(updated_token.owner, "Bob");
    }

    #[test]
    fn test_list_tokens() {
        let mut registry = AssetRegistry::new();
        
        let metadata1 = serde_json::json!({"type": "Car"});
        let metadata2 = serde_json::json!({"type": "Boat"});
        
        registry.create_token("Car".to_string(), "A fast car".to_string(), "Alice".to_string(), metadata1).unwrap();
        registry.create_token("Boat".to_string(), "A luxury yacht".to_string(), "Bob".to_string(), metadata2).unwrap();
        
        let tokens = registry.list_tokens();
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_token_exists() {
        let mut registry = AssetRegistry::new();
        
        let metadata = serde_json::json!({"type": "Painting"});
        let token_id = registry.create_token("Mona Lisa".to_string(), "A famous painting".to_string(), "Louvre".to_string(), metadata).unwrap();
        
        assert!(registry.token_exists(&token_id));
        assert!(!registry.token_exists("non_existent_token"));
    }

    #[test]
    fn test_transfer_non_existent_token() {
        let mut registry = AssetRegistry::new();
        
        let result = registry.transfer_token("non_existent_token", "Alice".to_string());
        assert!(result.is_err());
    }
}
