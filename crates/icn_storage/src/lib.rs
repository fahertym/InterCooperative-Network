use icn_common::{IcnResult, IcnError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, warn, error};
use sha2::{Sha256, Digest};

pub struct StorageManager {
    replication_factor: usize,
    nodes: Arc<RwLock<Vec<StorageNode>>>,
    data_location: Arc<RwLock<HashMap<String, Vec<usize>>>>,
}

struct StorageNode {
    id: usize,
    data: HashMap<String, Vec<u8>>,
}

impl StorageManager {
    pub fn new(replication_factor: usize) -> Self {
        StorageManager {
            replication_factor,
            nodes: Arc::new(RwLock::new(Vec::new())),
            data_location: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_node(&self) -> IcnResult<usize> {
        let mut nodes = self.nodes.write().map_err(|_| IcnError::Storage("Failed to lock nodes".into()))?;
        let node_id = nodes.len();
        nodes.push(StorageNode {
            id: node_id,
            data: HashMap::new(),
        });
        info!("Added new storage node with ID: {}", node_id);
        Ok(node_id)
    }

    pub fn store_data(&self, key: &str, value: Vec<u8>) -> IcnResult<()> {
        let nodes = self.nodes.read().map_err(|_| IcnError::Storage("Failed to lock nodes".into()))?;
        if nodes.is_empty() {
            return Err(IcnError::Storage("No storage nodes available".into()));
        }

        let node_count = nodes.len();
        let mut data_location = self.data_location.write().map_err(|_| IcnError::Storage("Failed to lock data location".into()))?;

        let selected_nodes = self.select_nodes(key, node_count);
        data_location.insert(key.to_string(), selected_nodes.clone());

        drop(nodes);
        drop(data_location);

        for &node_id in &selected_nodes {
            self.store_on_node(node_id, key, value.clone())?;
        }

        info!("Stored data with key: {} on {} nodes", key, selected_nodes.len());
        Ok(())
    }

    pub fn retrieve_data(&self, key: &str) -> IcnResult<Vec<u8>> {
        let data_location = self.data_location.read().map_err(|_| IcnError::Storage("Failed to lock data location".into()))?;
        let node_ids = data_location.get(key).ok_or_else(|| IcnError::Storage("Data not found".into()))?;

        for &node_id in node_ids {
            if let Ok(data) = self.retrieve_from_node(node_id, key) {
                return Ok(data);
            }
        }

        Err(IcnError::Storage("Failed to retrieve data from any node".into()))
    }

    pub fn delete_data(&self, key: &str) -> IcnResult<()> {
        let mut data_location = self.data_location.write().map_err(|_| IcnError::Storage("Failed to lock data location".into()))?;
        let node_ids = data_location.remove(key).ok_or_else(|| IcnError::Storage("Data not found".into()))?;

        for &node_id in &node_ids {
            self.delete_from_node(node_id, key)?;
        }

        info!("Deleted data with key: {} from {} nodes", key, node_ids.len());
        Ok(())
    }

    fn select_nodes(&self, key: &str, node_count: usize) -> Vec<usize> {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        let mut selected_nodes = Vec::new();
        let mut start = u64::from_be_bytes(hash[0..8].try_into().unwrap()) as usize;

        for _ in 0..self.replication_factor.min(node_count) {
            selected_nodes.push(start % node_count);
            start = start.wrapping_add(1);
        }

        selected_nodes
    }

    fn store_on_node(&self, node_id: usize, key: &str, value: Vec<u8>) -> IcnResult<()> {
        let mut nodes = self.nodes.write().map_err(|_| IcnError::Storage("Failed to lock nodes".into()))?;
        let node = nodes.get_mut(node_id).ok_or_else(|| IcnError::Storage("Node not found".into()))?;
        node.data.insert(key.to_string(), value);
        Ok(())
    }

    fn retrieve_from_node(&self, node_id: usize, key: &str) -> IcnResult<Vec<u8>> {
        let nodes = self.nodes.read().map_err(|_| IcnError::Storage("Failed to lock nodes".into()))?;
        let node = nodes.get(node_id).ok_or_else(|| IcnError::Storage("Node not found".into()))?;
        node.data.get(key).cloned().ok_or_else(|| IcnError::Storage("Data not found on node".into()))
    }

    fn delete_from_node(&self, node_id: usize, key: &str) -> IcnResult<()> {
        let mut nodes = self.nodes.write().map_err(|_| IcnError::Storage("Failed to lock nodes".into()))?;
        let node = nodes.get_mut(node_id).ok_or_else(|| IcnError::Storage("Node not found".into()))?;
        node.data.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve_data() {
        let storage_manager = StorageManager::new(3);
        for _ in 0..5 {
            storage_manager.add_node().unwrap();
        }

        let key = "test_key";
        let value = b"test_value".to_vec();

        assert!(storage_manager.store_data(key, value.clone()).is_ok());
        let retrieved_value = storage_manager.retrieve_data(key).unwrap();
        assert_eq!(retrieved_value, value);
    }

    #[test]
    fn test_delete_data() {
        let storage_manager = StorageManager::new(3);
        for _ in 0..5 {
            storage_manager.add_node().unwrap();
        }

        let key = "delete_test_key";
        let value = b"delete_test_value".to_vec();

        storage_manager.store_data(key, value).unwrap();
        assert!(storage_manager.delete_data(key).is_ok());
        assert!(storage_manager.retrieve_data(key).is_err());
    }

    #[test]
    fn test_replication() {
        let storage_manager = StorageManager::new(3);
        for _ in 0..5 {
            storage_manager.add_node().unwrap();
        }

        let key = "replication_test_key";
        let value = b"replication_test_value".to_vec();

        storage_manager.store_data(key, value.clone()).unwrap();

        let data_location = storage_manager.data_location.read().unwrap();
        let stored_nodes = data_location.get(key).unwrap();
        assert_eq!(stored_nodes.len(), 3);

        let node_to_fail = stored_nodes[0];
        {
            let mut nodes = storage_manager.nodes.write().unwrap();
            nodes[node_to_fail].data.clear();
        }

        let retrieved_value = storage_manager.retrieve_data(key).unwrap();
        assert_eq!(retrieved_value, value);
    }

    #[test]
    fn test_node_selection() {
        let storage_manager = StorageManager::new(3);
        for _ in 0..10 {
            storage_manager.add_node().unwrap();
        }

        let key1 = "test_key_1";
        let key2 = "test_key_2";
        let value = b"test_value".to_vec();

        storage_manager.store_data(key1, value.clone()).unwrap();
        storage_manager.store_data(key2, value.clone()).unwrap();

        let data_location = storage_manager.data_location.read().unwrap();
        let nodes1 = data_location.get(key1).unwrap();
        let nodes2 = data_location.get(key2).unwrap();

        assert_ne!(nodes1, nodes2);

        assert_eq!(nodes1.len(), 3);
        assert_eq!(nodes2.len(), 3);
    }

    #[test]
    fn test_insufficient_nodes() {
        let storage_manager = StorageManager::new(3);
        storage_manager.add_node().unwrap();
        storage_manager.add_node().unwrap();

        let key = "insufficient_nodes_key";
        let value = b"test_value".to_vec();

        assert!(storage_manager.store_data(key, value.clone()).is_ok());

        let data_location = storage_manager.data_location.read().unwrap();
        let stored_nodes = data_location.get(key).unwrap();
        assert_eq!(stored_nodes.len(), 2);

        let retrieved_value = storage_manager.retrieve_data(key).unwrap();
        assert_eq!(retrieved_value, value);
    }

    #[test]
    fn test_data_integrity() {
        let storage_manager = StorageManager::new(3);
        for _ in 0..5 {
            storage_manager.add_node().unwrap();
        }

        let key = "integrity_test_key";
        let value = b"integrity_test_value".to_vec();

        storage_manager.store_data(key, value.clone()).unwrap();

        let data_location = storage_manager.data_location.read().unwrap();
        let stored_nodes = data_location.get(key).unwrap();
        let corrupt_node = stored_nodes[0];

        {
            let mut nodes = storage_manager.nodes.write().unwrap();
            nodes[corrupt_node].data.insert(key.to_string(), b"corrupt_value".to_vec());
        }

        let retrieved_value = storage_manager.retrieve_data(key).unwrap();
        assert_eq!(retrieved_value, value);
    }
}
