use icn_core::error::{Error, Result};
use std::collections::HashMap;

pub struct NamingService {
    names: HashMap<String, String>,
}

impl NamingService {
    pub fn new() -> Self {
        NamingService {
            names: HashMap::new(),
        }
    }

    pub fn resolve(&self, name: &str) -> Result<String> {
        self.names.get(name)
            .cloned()
            .ok_or_else(|| Error::NetworkError("Name not found".to_string()))
    }

    pub fn register(&mut self, name: &str, address: &str) -> Result<()> {
        if self.names.contains_key(name) {
            return Err(Error::NetworkError("Name already registered".to_string()));
        }
        self.names.insert(name.to_string(), address.to_string());
        Ok(())
    }
}
