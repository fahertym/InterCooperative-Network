// File: crates/icn_storage/src/lib.rs

use icn_common::{IcnResult, IcnError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, warn, error};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageNode {
    id: String,
    data: HashMap<String, Vec<u8>>,
}

pub struct StorageManager {
    replication_factor: usize,
    nodes: Arc<RwLock<Vec<StorageNode>>>,
    data_location: Arc<RwLock<HashMap<String, Vec<usize>>>>,
}

impl StorageManager {
    pub fn new(replication_factor: usize) -> Self {
        StorageManager {
            replication_factor,
            nodes: Arc::new(RwLock::new(Vec::new())),
            data_location: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_node(&self, id: String) -> IcnResult<()> {
        let mut nodes = self.nodes.write().map_err(|_| IcnError::Storage("Failed to lock nodes".into()))?;
        nodes.push(StorageNode {
            id: id.clone(),
            data: HashMap::new(),
        });
        info!("Added new storage node with ID: {}", id);
        Ok(())
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

    pub fn remove_data(&self, key: &str) -> IcnResult<()> {
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

    pub fn get_node_count(&self) -> usize {
        self.nodes.read().unwrap().len()
    }

    pub fn get_data_distribution(&self) -> IcnResult<HashMap<String, Vec<String>>> {
        let nodes = self.nodes.read().map_err(|_| IcnError::Storage("Failed to lock nodes".into()))?;
        let data_location = self.data_location.read().map_err(|_| IcnError::Storage("Failed to lock data location".into()))?;
        
        let mut distribution = HashMap::new();
        for (key, node_ids) in data_location.iter() {
            let node_names: Vec<String> = node_ids.iter()
                .filter_map(|&id| nodes.get(id).map(|node| node.id.clone()))
                .collect();
            distribution.insert(key.clone(), node_names);
        }
        
        Ok(distribution)
    }

    // New helper function to check if a key exists in the storage
    pub fn key_exists(&self, key: &str) -> IcnResult<bool> {
        let data_location = self.data_location.read().map_err(|_| IcnError::Storage("Failed to lock data location".into()))?;
        Ok(data_location.contains_key(key))
    }

    // New helper function to get the total size of stored data
    pub fn get_total_storage_size(&self) -> IcnResult<usize> {
        let nodes = self.nodes.read().map_err(|_| IcnError::Storage("Failed to lock nodes".into()))?;
        Ok(nodes.iter().map(|node| node.data.values().map(|v| v.len()).sum::<usize>()).sum())
    }

    // New helper function to get the number of keys stored
    pub fn get_key_count(&self) -> IcnResult<usize> {
        let data_location = self.data_location.read().map_err(|_| IcnError::Storage("Failed to lock data location".into()))?;
        Ok(data_location.len())
    }

    // New helper function to list all keys
    pub fn list_keys(&self) -> IcnResult<Vec<String>> {
        let data_location = self.data_location.read().map_err(|_| IcnError::Storage("Failed to lock data location".into()))?;
        Ok(data_location.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve_data() {
        let storage_manager = StorageManager::new(3);
        for i in 0..5 {
            storage_manager.add_node(format!("node{}", i)).unwrap();
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
        for i in 0..5 {
            storage_manager.add_node(format!("node{}", i)).unwrap();
        }

        let key = "delete_test_key";
        let value = b"delete_test_value".to_vec();

        storage_manager.store_data(key, value).unwrap();
        assert!(storage_manager.remove_data(key).is_ok());
        assert!(storage_manager.retrieve_data(key).is_err());
    }

    #[test]
    fn test_replication() {
        let storage_manager = StorageManager::new(3);
        for i in 0..5 {
            storage_manager.add_node(format!("node{}", i)).unwrap();
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
        for i in 0..10 {
            storage_manager.add_node(format!("node{}", i)).unwrap();
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
        storage_manager.add_node("node1".to_string()).unwrap();
        storage_manager.add_node("node2".to_string()).unwrap();

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
    fn test_get_node_count() {
        let storage_manager = StorageManager::new(3);
        assert_eq!(storage_manager.get_node_count(), 0);

        for i in 0..5 {
            storage_manager.add_node(format!("node{}", i)).unwrap();
        }

        assert_eq!(storage_manager.get_node_count(), 5);
    }

    #[test]
    fn test_get_data_distribution() {
        let storage_manager = StorageManager::new(2);
        for i in 0..4 {
            storage_manager.add_node(format!("node{}", i)).unwrap();
        }

        let key1 = "key1";
        let key2 = "key2";
        let value = b"test_value".to_vec();

        storage_manager.store_data(key1, value.clone()).unwrap();
        storage_manager.store_data(key2, value.clone()).unwrap();

        let distribution = storage_manager.get_data_distribution().unwrap();
        
        assert_eq!(distribution.len(), 2);
        assert!(distribution.contains_key(key1));
        assert!(distribution.contains_key(key2));
        assert_eq!(distribution[key1].len(), 2);
        assert_eq!(distribution[key2].len(), 2);
    }

    #[test]
    fn test_key_exists() {
        let storage_manager = StorageManager::new(3);
        storage_manager.add_node("node1".to_string()).unwrap();

        let key = "test_key";
        let value = b"test_value".to_vec();

        assert!(!storage_manager.key_exists(key).unwrap());
        storage_manager.store_data(key, value).unwrap();
        assert!(storage_manager.key_exists(key).unwrap());
    }

    #[test]
    fn test_get_total_storage_size() {
        let storage_manager = StorageManager::new(3);
        storage_manager.add_node("node1".to_string()).unwrap();

        let key1 = "test_key1";
        let key2 = "test_key2";
        let value1 = b"test_value1".to_vec();
        let value2 = b"test_value2".to_vec();

        assert_eq!(storage_manager.get_total_storage_size().unwrap(), 0);

        storage_manager.store_data(key1, value1.clone()).unwrap();
        storage_manager.store_data(key2, value2.clone()).unwrap();

        let expected_size = value1.len() + value2.len();
        assert_eq!(storage_manager.get_total_storage_size().unwrap(), expected_size);
    }

    #[test]
    fn test_get_key_count() {
        let storage_manager = StorageManager::new(3);
        storage_manager.add_node("node1".to_string()).unwrap();

        assert_eq!(storage_manager.get_key_count().unwrap(), 0);

        storage_manager.store_data("key1", b"value1".to_vec()).unwrap();
        storage_manager.store_data("key2", b"value2".to_vec()).unwrap();
        storage_manager.store_data("key3", b"value3".to_vec()).unwrap();

        assert_eq!(storage_manager.get_key_count().unwrap(), 3);

        storage_manager.remove_data("key2").unwrap();

        assert_eq!(storage_manager.get_key_count().unwrap(), 2);
    }

    #[test]
    fn test_list_keys() {
        let storage_manager = StorageManager::new(3);
        storage_manager.add_node("node1".to_string()).unwrap();

        let keys = vec!["key1", "key2", "key3"];
        for key in &keys {
            storage_manager.store_data(key, b"value".to_vec()).unwrap();
        }

        let listed_keys = storage_manager.list_keys().unwrap();
        assert_eq!(listed_keys.len(), 3);
        for key in keys {
            assert!(listed_keys.contains(&key.to_string()));
        }
    }
}