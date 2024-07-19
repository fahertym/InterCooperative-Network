use std::collections::HashMap;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageNode {
    data: HashMap<String, Vec<u8>>,
}

impl StorageNode {
    pub fn new() -> Self {
        StorageNode {
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

    pub fn update(&mut self, hash: &str, new_content: Vec<u8>) -> Result<(), String> {
        if self.data.contains_key(hash) {
            let new_hash = self.calculate_hash(&new_content);
            if new_hash != hash {
                return Err("Update would change the hash, use store instead".to_string());
            }
            self.data.insert(hash.to_string(), new_content);
            Ok(())
        } else {
            Err("Hash not found".to_string())
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

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let mut node = StorageNode::new();
        let content = b"Hello, ICN!".to_vec();
        let hash = node.store(content.clone());
        assert_eq!(node.retrieve(&hash), Some(&content));
    }

    #[test]
    fn test_delete() {
        let mut node = StorageNode::new();
        let content = b"Delete me".to_vec();
        let hash = node.store(content);
        assert!(node.delete(&hash));
        assert!(!node.contains(&hash));
    }

    #[test]
    fn test_update() {
        let mut node = StorageNode::new();
        let content = b"Original content".to_vec();
        let hash = node.store(content);
        let new_content = b"Original content".to_vec(); // Same content, should work
        assert!(node.update(&hash, new_content).is_ok());
        let different_content = b"Different content".to_vec();
        assert!(node.update(&hash, different_content).is_err());
    }

    #[test]
    fn test_list_hashes() {
        let mut node = StorageNode::new();
        let hash1 = node.store(b"Content 1".to_vec());
        let hash2 = node.store(b"Content 2".to_vec());
        let hashes = node.list_hashes();
        assert!(hashes.contains(&hash1));
        assert!(hashes.contains(&hash2));
    }

    #[test]
    fn test_clear_and_len() {
        let mut node = StorageNode::new();
        node.store(b"Content 1".to_vec());
        node.store(b"Content 2".to_vec());
        assert_eq!(node.len(), 2);
        node.clear();
        assert!(node.is_empty());
    }
}