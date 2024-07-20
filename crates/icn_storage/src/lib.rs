use icn_common::{CommonError, CommonResult};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use hex;

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

    pub fn update(&mut self, hash: &str, new_content: Vec<u8>) -> CommonResult<()> {
        if let Some(existing_content) = self.data.get_mut(hash) {
            let new_hash = calculate_hash(&new_content);
            if new_hash != *hash {
                return Err(CommonError::StorageError("Update would change the hash, use store instead".into()));
            }
            *existing_content = new_content;
            Ok(())
        } else {
            Err(CommonError::StorageError("Hash not found".into()))
        }
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

    pub fn add_node(&mut self, node: StorageNode) -> CommonResult<()> {
        if self.nodes.contains_key(&node.id) {
            return Err(CommonError::StorageError("Node already exists".into()));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn remove_node(&mut self, node_id: &str) -> CommonResult<()> {
        if self.nodes.remove(node_id).is_none() {
            return Err(CommonError::StorageError("Node not found".into()));
        }
        Ok(())
    }

    pub fn store_data(&mut self, content: Vec<u8>) -> CommonResult<String> {
        if self.nodes.is_empty() {
            return Err(CommonError::StorageError("No storage nodes available".into()));
        }
        
        // Simple round-robin selection for now
        let node = self.nodes.values_mut().next().unwrap();
        let hash = node.store(content);
        Ok(hash)
    }

    pub fn retrieve_data(&self, hash: &str) -> CommonResult<Vec<u8>> {
        for node in self.nodes.values() {
            if let Some(data) = node.retrieve(hash) {
                return Ok(data.clone());
            }
        }
        Err(CommonError::StorageError("Data not found".into()))
    }

    pub fn delete_data(&mut self, hash: &str) -> CommonResult<()> {
        for node in self.nodes.values_mut() {
            if node.delete(hash) {
                return Ok(());
            }
        }
        Err(CommonError::StorageError("Data not found".into()))
    }

    pub fn update_data(&mut self, hash: &str, new_content: Vec<u8>) -> CommonResult<()> {
        for node in self.nodes.values_mut() {
            if node.contains(hash) {
                return node.update(hash, new_content);
            }
        }
        Err(CommonError::StorageError("Data not found".into()))
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
        // Tests should cover all operations to ensure the storage functionality works as expected
    }
}
