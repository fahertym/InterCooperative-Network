use icn_common::{Error, Result};
use std::collections::BTreeMap;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageNode {
    id: String,
    data: BTreeMap<String, Vec<u8>>,
}

impl StorageNode {
    pub fn new(id: String) -> Self {
        StorageNode {
            id,
            data: BTreeMap::new(),
        }
    }

    pub fn store(&mut self, content: Vec<u8>) -> String {
        let hash = calculate_hash(&content);
        self.data.insert(hash.clone(), content);
        hash
    }

    pub fn retrieve(&self, hash: &str) -> Option<&Vec<u8>> {
        self.data.get(hash)
    }

    pub fn delete(&mut self, hash: &str) -> bool {
        self.data.remove(hash).is_some()
    }

    pub fn update(&mut self, hash: &str, new_content: Vec<u8>) -> Result<()> {
        let new_hash = calculate_hash(&new_content);
        if new_hash != hash {
            return Err(Error::StorageError("Update would change the hash, use store instead".into()));
        }
        self.data.insert(hash.to_string(), new_content);
        Ok(())
    }

    pub fn list_hashes(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    pub fn contains(&self, hash: &str) -> bool {
        self.data.contains_key(hash)
    }
}

pub struct StorageManager {
    nodes: BTreeMap<String, StorageNode>,
}

impl StorageManager {
    pub fn new() -> Self {
        StorageManager {
            nodes: BTreeMap::new(),
        }
    }

    pub fn add_node(&mut self, node: StorageNode) -> Result<()> {
        if self.nodes.contains_key(&node.id) {
            return Err(Error::StorageError("Node already exists".into()));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn remove_node(&mut self, node_id: &str) -> Result<()> {
        if self.nodes.remove(node_id).is_none() {
            return Err(Error::StorageError("Node not found".into()));
        }
        Ok(())
    }

    pub fn store_data(&mut self, content: Vec<u8>) -> Result<String> {
        if self.nodes.is_empty() {
            return Err(Error::StorageError("No storage nodes available".into()));
        }
        
        // Simple round-robin selection for now
        let node = self.nodes.values_mut().next().unwrap();
        let hash = node.store(content);
        Ok(hash)
    }

    pub fn retrieve_data(&self, hash: &str) -> Result<Vec<u8>> {
        for node in self.nodes.values() {
            if let Some(data) = node.retrieve(hash) {
                return Ok(data.clone());
            }
        }
        Err(Error::StorageError("Data not found".into()))
    }

    pub fn delete_data(&mut self, hash: &str) -> Result<()> {
        for node in self.nodes.values_mut() {
            if node.delete(hash) {
                return Ok(());
            }
        }
        Err(Error::StorageError("Data not found".into()))
    }

    pub fn update_data(&mut self, hash: &str, new_content: Vec<u8>) -> Result<()> {
        for node in self.nodes.values_mut() {
            if node.contains(hash) {
                return node.update(hash, new_content);
            }
        }
        Err(Error::StorageError("Data not found".into()))
    }

    pub fn list_all_hashes(&self) -> Vec<String> {
        let mut all_hashes = Vec::new();
        for node in self.nodes.values() {
            all_hashes.extend(node.list_hashes());
        }
        all_hashes.sort_unstable();
        all_hashes.dedup();
        all_hashes
    }
}

fn calculate_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_operations() {
        let mut manager = StorageManager::new();
        let node1 = StorageNode::new("node1".to_string());
        let node2 = StorageNode::new("node2".to_string());

        manager.add_node(node1).unwrap();
        manager.add_node(node2).unwrap();

        let content = b"Test data".to_vec();
        let hash = manager.store_data(content.clone()).unwrap();

        let retrieved_data = manager.retrieve_data(&hash).unwrap();
        assert_eq!(retrieved_data, content);

        manager.update_data(&hash, b"Updated data".to_vec()).unwrap();
        let updated_data = manager.retrieve_data(&hash).unwrap();
        assert_eq!(updated_data, b"Updated data".to_vec());

        manager.delete_data(&hash).unwrap();
        assert!(manager.retrieve_data(&hash).is_err());

        let all_hashes = manager.list_all_hashes();
        assert!(all_hashes.is_empty());
    }
}