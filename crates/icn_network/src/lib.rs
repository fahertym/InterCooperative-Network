use icn_types::{IcnResult, IcnError, Node, NodeType};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::net::SocketAddr;

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

    pub fn add_node(&mut self, node: Node) -> IcnResult<()> {
        if self.nodes.contains_key(&node.id) {
            return Err(IcnError::Network("Node already exists".to_string()));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn remove_node(&mut self, node_id: &str) -> IcnResult<()> {
        if self.nodes.remove(node_id).is_none() {
            return Err(IcnError::Network("Node not found".to_string()));
        }
        self.routing_table.retain(|_, v| v != node_id);
        Ok(())
    }

    pub fn send_packet(&self, packet: Packet) -> IcnResult<()> {
        let next_hop = self.routing_table.get(&packet.destination)
            .ok_or_else(|| IcnError::Network("Destination not found in routing table".to_string()))?;
        let node = self.nodes.get(next_hop)
            .ok_or_else(|| IcnError::Network("Next hop node not found".to_string()))?;
        
        // In a real implementation, this would send the packet to the next hop
        println!("Sending packet to {}: {:?}", node.address, packet);
        Ok(())
    }

    pub fn update_routing(&mut self, destination: String, next_hop: String) -> IcnResult<()> {
        if !self.nodes.contains_key(&next_hop) {
            return Err(IcnError::Network("Next hop node not found".to_string()));
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

    pub fn broadcast(&self, packet: Packet) -> IcnResult<()> {
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

        let node1 = Node {
            id: "node1".to_string(),
            node_type: NodeType::PersonalDevice,
            address: "127.0.0.1:8000".to_string(),
        };
        let node2 = Node {
            id: "node2".to_string(),
            node_type: NodeType::CooperativeServer,
            address: "127.0.0.1:8001".to_string(),
        };

        assert!(network.add_node(node1.clone()).is_ok());
        assert!(network.add_node(node2.clone()).is_ok());

        assert_eq!(network.list_nodes().len(), 2);

        assert!(network.update_routing("dest1".to_string(), "node2".to_string()).is_ok());

        let packet = Packet {
            packet_type: PacketType::Data,
            source: "node1".to_string(),
            destination: "dest1".to_string(),
            content: vec![1, 2, 3, 4],
        };

        assert!(network.send_packet(packet).is_ok());

        assert!(network.remove_node("node1").is_ok());
        assert_eq!(network.list_nodes().len(), 1);

        // Test adding an existing node
        assert!(network.add_node(node2.clone()).is_err());

        // Test removing a non-existent node
        assert!(network.remove_node("non_existent_node").is_err());

        // Test updating routing with a non-existent node
        assert!(network.update_routing("dest2".to_string(), "non_existent_node".to_string()).is_err());

        // Test broadcast
        let broadcast_packet = Packet {
            packet_type: PacketType::Control,
            source: "node2".to_string(),
            destination: "broadcast".to_string(),
            content: vec![5, 6, 7, 8],
        };
        assert!(network.broadcast(broadcast_packet).is_ok());

        // Test network topology
        let topology = network.get_network_topology();
        assert_eq!(topology.len(), 1);
        assert!(topology.contains_key("node2"));
    }

    #[test]
    fn test_packet_types() {
        let data_packet = Packet {
            packet_type: PacketType::Data,
            source: "source".to_string(),
            destination: "destination".to_string(),
            content: vec![1, 2, 3],
        };

        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            source: "source".to_string(),
            destination: "destination".to_string(),
            content: vec![4, 5, 6],
        };

        let control_packet = Packet {
            packet_type: PacketType::Control,
            source: "source".to_string(),
            destination: "destination".to_string(),
            content: vec![7, 8, 9],
        };

        assert!(matches!(data_packet.packet_type, PacketType::Data));
        assert!(matches!(interest_packet.packet_type, PacketType::Interest));
        assert!(matches!(control_packet.packet_type, PacketType::Control));
    }
}