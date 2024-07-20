// File: icn_storage/src/lib.rs

use icn_types::{IcnResult, IcnError};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageNode {
    id: String,
    data: HashMap<String, Vec<u8>>,
}

impl StorageNode {
    pub fn new(id: String) -> Self {
        StorageNode {
            id,
            data: HashMap::new(),
        }
    }

    pub fn store(&mut self, content: Vec<u8>) -> String {
        let hash = self.calculate_hash(&content);
        self.data.insert(hash.clone(), content);
        hash
    }

    pub fn retrieve(&self, hash: &str) -> Option<&Vec<u8>> {
        self.data.get(hash)
    }

    pub fn delete(&mut self, hash: &str) -> bool {
        self.data.remove(hash).is_some()
    }

    pub fn update(&mut self, hash: &str, new_content: Vec<u8>) -> IcnResult<()> {
        if self.data.contains_key(hash) {
            let new_hash = self.calculate_hash(&new_content);
            if new_hash != hash {
                return Err(IcnError::Storage("Update would change the hash, use store instead".to_string()));
            }
            self.data.insert(hash.to_string(), new_content);
            Ok(())
        } else {
            Err(IcnError::Storage("Hash not found".to_string()))
        }
    }

    fn calculate_hash(&self, content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn list_hashes(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    pub fn contains(&self, hash: &str) -> bool {
        self.data.contains_key(hash)
    }
}

pub struct StorageManager {
    nodes: HashMap<String, StorageNode>,
}

impl StorageManager {
    pub fn new() -> Self {
        StorageManager {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: StorageNode) -> IcnResult<()> {
        if self.nodes.contains_key(&node.id) {
            return Err(IcnError::Storage("Node already exists".to_string()));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn remove_node(&mut self, node_id: &str) -> IcnResult<()> {
        if self.nodes.remove(node_id).is_none() {
            return Err(IcnError::Storage("Node not found".to_string()));
        }
        Ok(())
    }

    pub fn store_data(&mut self, content: Vec<u8>) -> IcnResult<String> {
        if self.nodes.is_empty() {
            return Err(IcnError::Storage("No storage nodes available".to_string()));
        }
        
        // Simple round-robin selection for now
        let node = self.nodes.values_mut().next().unwrap();
        let hash = node.store(content);
        Ok(hash)
    }

    pub fn retrieve_data(&self, hash: &str) -> IcnResult<Vec<u8>> {
        for node in self.nodes.values() {
            if let Some(data) = node.retrieve(hash) {
                return Ok(data.clone());
            }
        }
        Err(IcnError::Storage("Data not found".to_string()))
    }

    pub fn delete_data(&mut self, hash: &str) -> IcnResult<()> {
        for node in self.nodes.values_mut() {
            if node.delete(hash) {
                return Ok(());
            }
        }
        Err(IcnError::Storage("Data not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_node() {
        let mut node = StorageNode::new("node1".to_string());
        let content = b"Hello, ICN!".to_vec();
        let hash = node.store(content.clone());
        
        assert_eq!(node.retrieve(&hash), Some(&content));
        assert!(node.contains(&hash));
        assert!(node.delete(&hash));
        assert!(!node.contains(&hash));
    }

    #[test]
    fn test_storage_manager() {
        let mut manager = StorageManager::new();
        let node1 = StorageNode::new("node1".to_string());
        let node2 = StorageNode::new("node2".to_string());
        
        manager.add_node(node1).unwrap();
        manager.add_node(node2).unwrap();
        
        let content = b"Hello, ICN!".to_vec();
        let hash = manager.store_data(content.clone()).unwrap();
        
        assert_eq!(manager.retrieve_data(&hash).unwrap(), content);
        assert!(manager.delete_data(&hash).is_ok());
        assert!(manager.retrieve_data(&hash).is_err());
    }
}