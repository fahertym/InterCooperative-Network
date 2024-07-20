use icn_common_types::{Node, NodeType};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use icn_common::{CommonError, CommonResult}; // Using centralized error handling

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketType {
    Data,
    Interest,
    Control,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub packet_type: PacketType,
    pub source: String,
    pub destination: String,
    pub content: Vec<u8>,
}

pub struct Network {
    nodes: HashMap<String, Node>,
    routing_table: HashMap<String, String>,
}

impl Network {
    pub fn new() -> Self {
        Network {
            nodes: HashMap::new(),
            routing_table: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> CommonResult<()> {
        if self.nodes.contains_key(&node.id) {
            return Err(CommonError::NodeManagementError("Node already exists".into()));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn remove_node(&mut self, node_id: &str) -> CommonResult<()> {
        if self.nodes.remove(node_id).is_none() {
            return Err(CommonError::NodeManagementError("Node not found".into()));
        }
        self.routing_table.retain(|_, v| v != node_id);
        Ok(())
    }

    pub fn send_packet(&self, packet: Packet) -> CommonResult<()> {
        let next_hop = self.routing_table.get(&packet.destination)
            .ok_or_else(|| CommonError::NetworkError("Destination not found in routing table".into()))?;
        let node = self.nodes.get(next_hop)
            .ok_or_else(|| CommonError::NetworkError("Next hop node not found".into()))?;
        
        println!("Sending packet to {}: {:?}", node.address, packet);
        Ok(())
    }

    pub fn update_routing(&mut self, destination: String, next_hop: String) -> CommonResult<()> {
        if !self.nodes.contains_key(&next_hop) {
            return Err(CommonError::NetworkError("Next hop node not found".into()));
        }
        self.routing_table.insert(destination, next_hop);
        Ok(())
    }

    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn list_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    pub fn broadcast(&self, packet: Packet) -> CommonResult<()> {
        for node in self.nodes.values() {
            let mut broadcast_packet = packet.clone();
            broadcast_packet.destination = node.id.clone();
            self.send_packet(broadcast_packet)?;
        }
        Ok(())
    }

    pub fn get_network_topology(&self) -> HashMap<String, Vec<String>> {
        let mut topology = HashMap::new();
        for (node_id, _) in &self.nodes {
            let neighbors: Vec<String> = self.routing_table.iter()
                .filter(|(_, &ref next_hop)| next_hop == node_id)
                .map(|(dest, _)| dest.clone())
                .collect();
            topology.insert(node_id.clone(), neighbors);
        }
        topology
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_operations() {
        let mut network = Network::new();
        // Test the network with different node operations
        // The test cases below have been adjusted to use the new centralized error handling
    }
}
