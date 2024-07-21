use icn_common::{IcnResult, IcnError};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use log::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketType {
    Ping,
    Pong,
    PeerDiscovery,
    Transaction,
    Block,
    Consensus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub packet_type: PacketType,
    pub source: SocketAddr,
    pub destination: SocketAddr,
    pub content: Vec<u8>,
}

#[derive(Debug)]
pub struct Peer {
    pub address: SocketAddr,
    pub last_seen: std::time::Instant,
}

pub struct Network {
    local_addr: SocketAddr,
    peers: HashMap<SocketAddr, Peer>,
    pending_connections: HashSet<SocketAddr>,
    packet_sender: mpsc::Sender<Packet>,
    packet_receiver: mpsc::Receiver<Packet>,
}

impl Network {
    pub fn new(local_addr: SocketAddr) -> Self {
        let (packet_sender, packet_receiver) = mpsc::channel(100);
        Network {
            local_addr,
            peers: HashMap::new(),
            pending_connections: HashSet::new(),
            packet_sender,
            packet_receiver,
        }
    }

    pub async fn start(&mut self) -> IcnResult<()> {
        info!("Starting network on {}", self.local_addr);
        // TODO: Implement actual network initialization
        Ok(())
    }

    pub async fn stop(&mut self) -> IcnResult<()> {
        info!("Stopping network");
        // TODO: Implement network shutdown
        Ok(())
    }

    pub async fn connect_to_peer(&mut self, peer_addr: SocketAddr) -> IcnResult<()> {
        if self.peers.contains_key(&peer_addr) || self.pending_connections.contains(&peer_addr) {
            return Ok(());
        }

        self.pending_connections.insert(peer_addr);
        // TODO: Implement actual peer connection logic
        info!("Connecting to peer: {}", peer_addr);
        
        // Simulating successful connection
        self.add_peer(peer_addr)?;
        self.pending_connections.remove(&peer_addr);
        Ok(())
    }

    pub fn add_peer(&mut self, peer_addr: SocketAddr) -> IcnResult<()> {
        if self.peers.contains_key(&peer_addr) {
            return Err(IcnError::Network("Peer already connected".into()));
        }

        let peer = Peer {
            address: peer_addr,
            last_seen: std::time::Instant::now(),
        };
        self.peers.insert(peer_addr, peer);
        info!("Added new peer: {}", peer_addr);
        Ok(())
    }

    pub fn remove_peer(&mut self, peer_addr: &SocketAddr) -> IcnResult<()> {
        if self.peers.remove(peer_addr).is_none() {
            return Err(IcnError::Network("Peer not found".into()));
        }
        info!("Removed peer: {}", peer_addr);
        Ok(())
    }

    pub async fn send_packet(&self, packet: Packet) -> IcnResult<()> {
        self.packet_sender.send(packet).await.map_err(|e| IcnError::Network(format!("Failed to send packet: {}", e)))
    }

    pub async fn receive_packet(&mut self) -> IcnResult<Packet> {
        self.packet_receiver.recv().await.ok_or(IcnError::Network("Failed to receive packet".into()))
    }

    pub async fn broadcast(&self, packet_type: PacketType, content: Vec<u8>) -> IcnResult<()> {
        for peer_addr in self.peers.keys() {
            let packet = Packet {
                packet_type: packet_type.clone(),
                source: self.local_addr,
                destination: *peer_addr,
                content: content.clone(),
            };
            self.send_packet(packet).await?;
        }
        Ok(())
    }

    pub fn get_peers(&self) -> Vec<SocketAddr> {
        self.peers.keys().cloned().collect()
    }

    pub async fn handle_incoming_packets(&mut self) -> IcnResult<()> {
        while let Ok(packet) = self.receive_packet().await {
            match packet.packet_type {
                PacketType::Ping => self.handle_ping(packet).await?,
                PacketType::Pong => self.handle_pong(packet).await?,
                PacketType::PeerDiscovery => self.handle_peer_discovery(packet).await?,
                PacketType::Transaction => self.handle_transaction(packet).await?,
                PacketType::Block => self.handle_block(packet).await?,
                PacketType::Consensus => self.handle_consensus(packet).await?,
            }
        }
        Ok(())
    }

    async fn handle_ping(&self, packet: Packet) -> IcnResult<()> {
        let pong = Packet {
            packet_type: PacketType::Pong,
            source: self.local_addr,
            destination: packet.source,
            content: vec![],
        };
        self.send_packet(pong).await
    }

    async fn handle_pong(&mut self, packet: Packet) -> IcnResult<()> {
        if let Some(peer) = self.peers.get_mut(&packet.source) {
            peer.last_seen = std::time::Instant::now();
        }
        Ok(())
    }

    async fn handle_peer_discovery(&mut self, packet: Packet) -> IcnResult<()> {
        let peer_addresses: Vec<SocketAddr> = bincode::deserialize(&packet.content)
            .map_err(|e| IcnError::Network(format!("Failed to deserialize peer addresses: {}", e)))?;

        for addr in peer_addresses {
            if addr != self.local_addr && !self.peers.contains_key(&addr) {
                self.connect_to_peer(addr).await?;
            }
        }

        // Share our known peers with the sender
        let our_peers: Vec<SocketAddr> = self.peers.keys().cloned().collect();
        let response = Packet {
            packet_type: PacketType::PeerDiscovery,
            source: self.local_addr,
            destination: packet.source,
            content: bincode::serialize(&our_peers)
                .map_err(|e| IcnError::Network(format!("Failed to serialize peer addresses: {}", e)))?,
        };
        self.send_packet(response).await
    }

    async fn handle_transaction(&self, packet: Packet) -> IcnResult<()> {
        // TODO: Implement transaction handling logic
        // For now, we'll just log the received transaction
        info!("Received transaction from {}", packet.source);
        Ok(())
    }

    async fn handle_block(&self, packet: Packet) -> IcnResult<()> {
        // TODO: Implement block handling logic
        // For now, we'll just log the received block
        info!("Received block from {}", packet.source);
        Ok(())
    }

    async fn handle_consensus(&self, packet: Packet) -> IcnResult<()> {
        // TODO: Implement consensus message handling logic
        // For now, we'll just log the received consensus message
        info!("Received consensus message from {}", packet.source);
        Ok(())
    }

    pub async fn maintain_network(&mut self) -> IcnResult<()> {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            self.remove_inactive_peers().await?;
            self.discover_new_peers().await?;
        }
    }

    async fn remove_inactive_peers(&mut self) -> IcnResult<()> {
        let now = std::time::Instant::now();
        let inactive_peers: Vec<SocketAddr> = self.peers
            .iter()
            .filter(|(_, peer)| now.duration_since(peer.last_seen) > std::time::Duration::from_secs(300))
            .map(|(addr, _)| *addr)
            .collect();

        for addr in inactive_peers {
            self.remove_peer(&addr)?;
        }
        Ok(())
    }

    async fn discover_new_peers(&mut self) -> IcnResult<()> {
        let peer_discovery_packet = Packet {
            packet_type: PacketType::PeerDiscovery,
            source: self.local_addr,
            destination: self.local_addr, // This will be replaced for each peer
            content: bincode::serialize(&self.get_peers())
                .map_err(|e| IcnError::Network(format!("Failed to serialize peer addresses: {}", e)))?,
        };

        for peer_addr in self.get_peers() {
            let mut packet = peer_discovery_packet.clone();
            packet.destination = peer_addr;
            self.send_packet(packet).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_network_operations() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut network = Network::new("127.0.0.1:8000".parse().unwrap());
            
            // Test starting the network
            assert!(network.start().await.is_ok());

            // Test adding a peer
            let peer_addr: SocketAddr = "127.0.0.1:8001".parse().unwrap();
            assert!(network.connect_to_peer(peer_addr).await.is_ok());
            assert_eq!(network.get_peers().len(), 1);

            // Test removing a peer
            assert!(network.remove_peer(&peer_addr).is_ok());
            assert_eq!(network.get_peers().len(), 0);

            // Test sending a packet
            let packet = Packet {
                packet_type: PacketType::Ping,
                source: network.local_addr,
                destination: peer_addr,
                content: vec![],
            };
            assert!(network.send_packet(packet).await.is_ok());

            // Test stopping the network
            assert!(network.stop().await.is_ok());
        });
    }
}