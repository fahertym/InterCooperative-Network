use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::storage_node::StorageNode;

#[derive(Debug)]
pub struct StorageManager {
    nodes: HashMap<Uuid, Arc<Mutex<StorageNode>>>,
}

impl StorageManager {
    pub fn new() -> Self {
        StorageManager {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Arc<Mutex<StorageNode>>) -> Uuid {
        let id = Uuid::new_v4();
        self.nodes.insert(id, node);
        id
    }

    pub fn remove_node(&mut self, id: Uuid) {
        self.nodes.remove(&id);
    }

    pub fn get_node(&self, id: &Uuid) -> Option<Arc<Mutex<StorageNode>>> {
        self.nodes.get(id).cloned()
    }

    pub fn list_nodes(&self) -> Vec<Uuid> {
        self.nodes.keys().cloned().collect()
    }
}
