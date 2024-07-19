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
            .cl