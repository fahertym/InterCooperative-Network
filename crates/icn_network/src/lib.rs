mod node;
mod packet;
mod routing;
mod protocol;
mod security;
mod discovery;
mod naming;

pub use node::{Node, NodeType};
pub use packet::{Packet, PacketType};
pub use routing::RoutingTable;
pub use protocol::NetworkProtocol;
pub use security::SecurityManager;
pub use discovery::DiscoveryService;
pub use naming::NamingService;

use icn_core::error::{Error, Result};

pub struct Network {
    nodes: Vec<Node>,
    routing_table: RoutingTable,
    protocol: NetworkProtocol,
    security_manager: SecurityManager,
    discovery_service: DiscoveryService,
    naming_service: NamingService,
}

impl Network {
    pub fn new() -> Self {
        Network {
            nodes: Vec::new(),
            routing_table: RoutingTable::new(),
            protocol: NetworkProtocol::new(),
            security_manager: SecurityManager::new(),
            discovery_service: DiscoveryService::new(),
            naming_service: NamingService::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> Result<()> {
        self.nodes.push(node);
        self.discovery_service.register_node(&node)?;
        self.routing_table.update(&self.nodes)?;
        Ok(())
    }

    pub fn remove_node(&mut self, node_id: &str) -> Result<()> {
        self.nodes.retain(|n| n.id != node_id);
        self.discovery_service.unregister_node(node_id)?;
        self.routing_table.update(&self.nodes)?;
        Ok(())
    }

    pub fn send_packet(&self, packet: Packet) -> Result<()> {
        self.security_manager.validate_packet(&packet)?;
        let route = self.routing_table.get_route(&packet.destination)?;
        self.protocol.send_packet(&packet, &route)
    }

    pub fn receive_packet(&self, packet: Packet) -> Result<()> {
        self.security_manager.validate_packet(&packet)?;
        self.protocol.process_packet(packet)
    }

    pub fn resolve_name(&self, name: &str) -> Result<String> {
        self.naming_service.resolve(name)
    }

    pub fn register_name(&mut self, name: &str, address: &str) -> Result<()> {
        self.naming_service.register(name, address)
    }
}