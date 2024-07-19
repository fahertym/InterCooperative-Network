use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::blockchain::Block;
use super::node::Node;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Network {
    nodes: HashMap<String, Node>,
}

impl Network {
    pub fn new() -> Self {
        Network {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.remove(node_id);
    }

    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn broadcast_block(&self, block: &Block) {
        println!("Broadcasting block {} to all nodes", block.index);
        // Actual implementation would involve network communication
    }

    pub fn synchronize_blockchain(&self, _blockchain: &[Block]) {
        // Implement synchronization logic here
    }

    pub fn get_all_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PacketType {
    Interest,
    Data,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Packet {
    pub packet_type: PacketType,
    pub name: String,
    pub content: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::node::NodeType;

    #[test]
    fn test_network_operations() {
        let mut network = Network::new();

        let node1 = Node::new("node1", NodeType::PersonalDevice, "192.168.1.1");
        let node2 = Node::new("node2", NodeType::CooperativeServer, "192.168.1.2");
        network.add_node(node1.clone());
        network.add_node(node2.clone());

        assert_eq!(network.node_count(), 2);

        let retrieved_node = network.get_node("node1").unwrap();
        assert_eq!(retrieved_node.id, "node1");
        assert_eq!(retrieved_node.address, "192.168.1.1");

        network.remove_node("node1");
        assert_eq!(network.node_count(), 1);
        assert!(network.get_node("node1").is_none());

        let block = Block {
            index: 1,
            timestamp: 0,
            transactions: vec![],
            previous_hash: "previous_hash".to_string(),
            hash: "hash".to_string(),
            nonce: 0,
            gas_used: 0,
            smart_contract_results: HashMap::new(),
        };
        network.broadcast_block(&block);

        network.synchronize_blockchain(&[block]);
    }

    #[test]
    fn test_packet_creation() {
        let packet = Packet {
            packet_type: PacketType::Interest,
            name: "test_packet".to_string(),
            content: vec![1, 2, 3, 4],
        };

        assert_eq!(packet.name, "test_packet");
        assert_eq!(packet.content, vec![1, 2, 3, 4]);

        match packet.packet_type {
            PacketType::Interest => assert!(true),
            _ => assert!(false, "Unexpected packet type"),
        }
    }
}