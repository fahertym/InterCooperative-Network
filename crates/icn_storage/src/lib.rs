pub mod storage_manager;
pub mod storage_node;

use crate::storage_manager::StorageManager;
use crate::storage_node::StorageNode;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use log::info;
use uuid::Uuid;

pub struct ICNStorage {
    storage_manager: Arc<Mutex<StorageManager>>,
    storage_nodes: Vec<Arc<Mutex<StorageNode>>>,
}

impl ICNStorage {
    pub fn new(storage_manager: StorageManager) -> Self {
        ICNStorage {
            storage_manager: Arc::new(Mutex::new(storage_manager)),
            storage_nodes: Vec::new(),
        }
    }

    pub fn add_storage_node(&mut self, storage_node: StorageNode) {
        self.storage_nodes.push(Arc::new(Mutex::new(storage_node)));
    }

    pub fn get_storage_manager(&self) -> Arc<Mutex<StorageManager>> {
        Arc::clone(&self.storage_manager)
    }

    pub fn get_storage_nodes(&self) -> Vec<Arc<Mutex<StorageNode>>> {
        self.storage_nodes.clone()
    }
}
