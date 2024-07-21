use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug)]
pub struct StorageManager {
    nodes: Arc<RwLock<HashMap<String, Arc<RwLock<StorageNode>>>>>,
    replication_factor: usize,
}

impl StorageManager {
    pub fn new(replication_factor: usize) -> Self {
        StorageManager {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            replication_factor,
        }
    }

    pub async fn add_node(&self, node: StorageNode) -> Result<(), String> {
        let mut nodes = self.nodes.write().map_err(|e| e.to_string())?;
        nodes.insert(node.id.clone(), Arc::new(RwLock::new(node)));
        Ok(())
    }

    pub async fn store_data(&self, content: Vec<u8>) -> Result<String, String> {
        let hash = calculate_hash(&content);
        let nodes = self.nodes.read().map_err(|e| e.to_string())?;
        
        if nodes.len() < self.replication_factor {
            return Err("Not enough nodes for replication".to_string());
        }

        let selected_nodes: Vec<_> = nodes.values().take(self.replication_factor).collect();

        for node in selected_nodes {
            let mut node = node.write().map_err(|e| e.to_string())?;
            node.store(content.clone());
        }

        Ok(hash)
    }

    pub async fn retrieve_data(&self, hash: &str) -> Result<Option<Vec<u8>>, String> {
        let nodes = self.nodes.read().map_err(|e| e.to_string())?;

        for node in nodes.values() {
            let node = node.read().map_err(|e| e.to_string())?;
            if let Some(data) = node.retrieve(hash) {
                return Ok(Some(data.clone()));
            }
        }

        Ok(None)
    }

    pub async fn delete_data(&self, hash: &str) -> Result<bool, String> {
        let nodes = self.nodes.read().map_err(|e| e.to_string())?;
        let mut deleted = false;

        for node in nodes.values() {
            let mut node = node.write().map_err(|e| e.to_string())?;
            if node.delete(hash) {
                deleted = true;
            }
        }

        Ok(deleted)
    }
}

fn calculate_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_operations() {
        let manager = StorageManager::new(2);

        manager.add_node(StorageNode::new("node1".to_string())).await.unwrap();
        manager.add_node(StorageNode::new("node2".to_string())).await.unwrap();

        let content = b"Test data".to_vec();
        let hash = manager.store_data(content.clone()).await.unwrap();

        let retrieved_data = manager.retrieve_data(&hash).await.unwrap().unwrap();
        assert_eq!(retrieved_data, content);

        let deleted = manager.delete_data(&hash).await.unwrap();
        assert!(deleted);

        let not_found = manager.retrieve_data(&hash).await.unwrap();
        assert!(not_found.is_none());
    }
}