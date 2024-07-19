use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    PersonalDevice,
    CooperativeServer,
    GovernmentServer,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub address: String,
}

impl Node {
    pub fn new(id: &str, node_type: NodeType, address: &str) -> Self {
        Node {
            id: id.to_string(),
            node_type,
            address: address.to_string(),
        }
    }
}
