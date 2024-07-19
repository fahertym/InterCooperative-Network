use serde::{Serialize, Deserialize};

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

impl Packet {
    pub fn new(packet_type: PacketType, source: String, destination: String, content: Vec<u8>) -> Self {
        Packet {
            packet_type,
            source,
            destination,
            content,
        }
    }
}