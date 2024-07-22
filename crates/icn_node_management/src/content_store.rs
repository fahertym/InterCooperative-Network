use icn_common::{IcnError, IcnResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, error};

/// Represents a content store in the ICN project.
pub struct ContentStore {
    store: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl ContentStore {
    /// Creates a new instance of ContentStore.
    pub fn new() -> Self {
        ContentStore {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Stores content in the content store.
    ///
    /// # Arguments
    ///
    /// * `content_id` - The ID of the content.
    /// * `data` - The content data.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn store(&self, content_id: String, data: Vec<u8>) -> IcnResult<()> {
        let mut store = self.store.write().unwrap();
        store.insert(content_id.clone(), data);
        info!("Content stored with ID: {}", content_id);
        Ok(())
    }

    /// Retrieves content from the content store.
    ///
    /// # Arguments
    ///
    /// * `content_id` - The ID of the content to be retrieved.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Vec<u8>>` - The retrieved content data.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn retrieve(&self, content_id: &str) -> IcnResult<Vec<u8>> {
        let store = self.store.read().unwrap();
        store.get(content_id).cloned().ok_or_else(|| IcnError::Storage("Content not found".into()))
    }

    /// Removes content from the content store.
    ///
    /// # Arguments
    ///
    /// * `content_id` - The ID of the content to be removed.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn remove(&self, content_id: &str) -> IcnResult<()> {
        let mut store = self.store.write().unwrap();
        store.remove(content_id);
        info!("Content removed with ID: {}", content_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve_content() {
        let content_store = ContentStore::new();
        let content_id = "content_1".to_string();
        let data = vec![1, 2, 3];
        assert!(content_store.store(content_id.clone(), data.clone()).is_ok());
        let retrieved_data = content_store.retrieve(&content_id).unwrap();
        assert_eq!(retrieved_data, data);
    }

    #[test]
    fn test_retrieve_non_existent_content() {
        let content_store = ContentStore::new();
        assert!(content_store.retrieve("non_existent_content").is_err());
    }

    #[test]
    fn test_remove_content() {
        let content_store = ContentStore::new();
        let content_id = "content_1".to_string();
        let data = vec![1, 2, 3];
        content_store.store(content_id.clone(), data).unwrap();
        assert!(content_store.remove(&content_id).is_ok());
        assert!(content_store.retrieve(&content_id).is_err());
    }
}
