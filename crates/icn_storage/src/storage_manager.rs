use crate::storage_node::StorageNode;
use icn_common::{IcnError, IcnResult};
use std::sync::{Arc, RwLock};
use log::{info, error};

/// Manages storage nodes and handles data storage operations.
pub struct StorageManager {
    nodes: Arc<RwLock<Vec<StorageNode>>>,
}

impl StorageManager {
    /// Creates a new instance of StorageManager.
    pub fn new() -> Self {
        StorageManager {
            nodes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Adds a storage node to the manager.
    ///
    /// # Arguments
    ///
    /// * `node` - The storage node to be added.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn add_node(&self, node: StorageNode) -> IcnResult<()> {
        let mut nodes = self.nodes.write().unwrap();
        nodes.push(node);
        info!("Storage node added");
        Ok(())
    }

    /// Removes a storage node from the manager.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the storage node to be removed.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn remove_node(&self, node_id: &str) -> IcnResult<()> {
        let mut nodes = self.nodes.write().unwrap();
        nodes.retain(|node| node.id != node_id);
        info!("Storage node removed");
        Ok(())
    }

    /// Stores data across available storage nodes.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be stored.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn store_data(&self, data: Vec<u8>) -> IcnResult<()> {
        let nodes = self.nodes.read().unwrap();
        if nodes.is_empty() {
            return Err(IcnError::Storage("No available storage nodes".into()));
        }
        // Simple round-robin distribution of data
        let node = &nodes[0];
        node.store(data)?;
        info!("Data stored");
        Ok(())
    }

    /// Retrieves data from storage nodes.
    ///
    /// # Arguments
    ///
    /// * `data_id` - The ID of the data to be retrieved.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Vec<u8>>` - The retrieved data.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn retrieve_data(&self, data_id: &str) -> IcnResult<Vec<u8>> {
        let nodes = self.nodes.read().unwrap();
        for node in nodes.iter() {
            if let Ok(data) = node.retrieve(data_id) {
                return Ok(data);
            }
        }
        Err(IcnError::Storage("Data not found".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage_node::MockStorageNode;

    #[test]
    fn test_add_and_remove_node() {
        let manager = StorageManager::new();
        let node = MockStorageNode::new("Node1".to_string());
        assert!(manager.add_node(node).is_ok());
        assert!(manager.remove_node("Node1").is_ok());
    }

    #[test]
    fn test_store_and_retrieve_data() {
        let manager = StorageManager::new();
        let node = MockStorageNode::new("Node1".to_string());
        manager.add_node(node).unwrap();

        let data = vec![1, 2, 3];
        assert!(manager.store_data(data.clone()).is_ok());
        let retrieved_data = manager.retrieve_data("data_id").unwrap();
        assert_eq!(retrieved_data, data);
    }
}
