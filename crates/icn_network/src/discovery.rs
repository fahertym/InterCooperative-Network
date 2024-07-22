use icn_common::{IcnError, IcnResult};
use std::sync::{Arc, RwLock};
use log::{info, error};

/// Represents a node discovery mechanism in the network.
pub struct NodeDiscovery {
    discovered_nodes: Arc<RwLock<Vec<String>>>,
}

impl NodeDiscovery {
    /// Creates a new instance of NodeDiscovery.
    pub fn new() -> Self {
        NodeDiscovery {
            discovered_nodes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Starts the node discovery process.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn start(&self) -> IcnResult<()> {
        info!("Node discovery started");
        // Simulated discovery process
        let nodes = vec!["Node1".to_string(), "Node2".to_string()];
        let mut discovered_nodes = self.discovered_nodes.write().unwrap();
        *discovered_nodes = nodes;
        Ok(())
    }

    /// Stops the node discovery process.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn stop(&self) -> IcnResult<()> {
        info!("Node discovery stopped");
        let mut discovered_nodes = self.discovered_nodes.write().unwrap();
        discovered_nodes.clear();
        Ok(())
    }

    /// Retrieves the list of discovered nodes.
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - A vector containing the discovered nodes.
    pub fn get_discovered_nodes(&self) -> Vec<String> {
        let discovered_nodes = self.discovered_nodes.read().unwrap();
        discovered_nodes.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_discovery() {
        let discovery = NodeDiscovery::new();
        assert!(discovery.start().is_ok());
        assert_eq!(discovery.get_discovered_nodes().len(), 2);
        assert!(discovery.stop().is_ok());
        assert!(discovery.get_discovered_nodes().is_empty());
    }
}
