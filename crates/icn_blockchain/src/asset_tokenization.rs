use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
    tokens: Vec<AssetToken>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        AssetRegistry {
            tokens: Vec::new(),
        }
    }

    pub fn create_token(&mut self, name: String, description: String, owner: String, metadata: serde_json::Value) -> AssetToken {
        let token = AssetToken::new(name, description, owner, metadata);
        self.tokens.push(token.clone());
        token
    }

    pub fn transfer_token(&mut self, token_id: &str, new_owner: String) -> Result<(), String> {
        if let Some(token) = self.tokens.iter_mut().find(|t| t.id == token_id) {
            token.transfer(new_owner);
            Ok(())
        } else {
            Err("Token not found".to_string())
        }
    }

    pub fn get_token(&self, token_id: &str) -> Option<&AssetToken> {
        self.tokens.iter().find(|t| t.id == token_id)
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
}