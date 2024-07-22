use icn_common::{IcnError, IcnResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, error};

/// Represents a storage node in the ICN project.
pub struct StorageNode {
    pub id: String,
    data_store: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl StorageNode {
    /// Creates a new instance of StorageNode.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the storage node.
    ///
    /// # Returns
    ///
    /// * `StorageNode` - A new instance of StorageNode.
    pub fn new(id: String) -> Self {
        StorageNode {
            id,
            data_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Stores data in the storage node.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be stored.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn store(&self, data: Vec<u8>) -> IcnResult<()> {
        let mut data_store = self.data_store.write().unwrap();
        let data_id = format!("data_{}", data_store.len() + 1);
        data_store.insert(data_id.clone(), data);
        info!("Data stored with ID: {}", data_id);
        Ok(())
    }

    /// Retrieves data from the storage node.
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
    pub fn retrieve(&self, data_id: &str) -> IcnResult<Vec<u8>> {
        let data_store = self.data_store.read().unwrap();
        data_store.get(data_id).cloned().ok_or_else(|| IcnError::Storage("Data not found".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve_data() {
        let node = StorageNode::new("Node1".to_string());
        let data = vec![1, 2, 3];
        assert!(node.store(data.clone()).is_ok());
        let retrieved_data = node.retrieve("data_1").unwrap();
        assert_eq!(retrieved_data, data);
    }

    #[test]
    fn test_retrieve_non_existent_data() {
        let node = StorageNode::new("Node1".to_string());
        assert!(node.retrieve("non_existent_data").is_err());
    }
}
