use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NameRecord {
    pub name: String,
    pub owner: String,
    pub target: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct NamingSystem {
    records: HashMap<String, NameRecord>,
}

impl NamingSystem {
    pub fn new() -> Self {
        NamingSystem {
            records: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, owner: String, target: String, duration: chrono::Duration) -> Result<(), String> {
        if self.records.contains_key(&name) {
            return Err("Name already registered".to_string());
        }

        let now = Utc::now();
        let record = NameRecord {
            name: name.clone(),
            owner,
            target,
            created_at: now,
            expires_at: now + duration,
        };

        self.records.insert(name, record);
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<&NameRecord> {
        self.records.get(name)
    }

    pub fn update(&mut self, name: &str, new_target: String, owner: &str) -> Result<(), String> {
        if let Some(record) = self.records.get_mut(name) {
            if record.owner != owner {
                return Err("Not the owner of the name".to_string());
            }
            record.target = new_target;
            Ok(())
        } else {
            Err("Name not found".to_string())
        }
    }

    pub fn transfer(&mut self, name: &str, new_owner: String, current_owner: &str) -> Result<(), String> {
        if let Some(record) = self.records.get_mut(name) {
            if record.owner != current_owner {
                return Err("Not the owner of the name".to_string());
            }
            record.owner = new_owner;
            Ok(())
        } else {
            Err("Name not found".to_string())
        }
    }

    pub fn renew(&mut self, name: &str, duration: chrono::Duration, owner: &str) -> Result<(), String> {
        if let Some(record) = self.records.get_mut(name) {
            if record.owner != owner {
                return Err("Not the owner of the name".to_string());
            }
            record.expires_at = record.expires_at + duration;
            Ok(())
        } else {
            Err("Name not found".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naming_system() {
        let mut naming_system = NamingSystem::new();

        // Register a name
        naming_system.register(
            "alice.icn".to_string(),
            "Alice".to_string(),
            "0x1234...".to_string(),
            chrono::Duration::days(365)
        ).unwrap();

        // Resolve the name
        let record = naming_system.resolve("alice.icn").unwrap();
        assert_eq!(record.owner, "Alice");
        assert_eq!(record.target, "0x1234...");

        // Update the target
        naming_system.update("alice.icn", "0x5678...".to_string(), "Alice").unwrap();
        let updated_record = naming_system.resolve("alice.icn").unwrap();
        assert_eq!(updated_record.target, "0x5678...");

        // Transfer ownership
        naming_system.transfer("alice.icn", "Bob".to_string(), "Alice").unwrap();
        let transferred_record = naming_system.resolve("alice.icn").unwrap();
        assert_eq!(transferred_record.owner, "Bob");

        // Attempt to update with old owner (should fail)
        assert!(naming_system.update("alice.icn", "0x9876...".to_string(), "Alice").is_err());
    }
}