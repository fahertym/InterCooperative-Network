use icn_common::{IcnResult, IcnError, Transaction, Block};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use log::{info, warn, error};

pub struct Network {
    local_addr: SocketAddr,
    peers: HashMap<SocketAddr, PeerInfo>,
    start_time: Option<Instant>,
    event_sender: mpsc::Sender<NetworkEvent>,
    event_receiver: mpsc::Receiver<NetworkEvent>,
}

struct PeerInfo {
    last_seen: Instant,
}

#[derive(Debug)]
pub enum NetworkEvent {
    NewTransaction(Transaction),
    NewBlock(Block),
    PeerConnected(SocketAddr),
    PeerDisconnected(SocketAddr),
}

impl Network {
    pub fn new(local_addr: SocketAddr) -> Self {
        let (event_sender, event_receiver) = mpsc::channel(100); // Adjust buffer size as needed
        Network {
            local_addr,
            peers: HashMap::new(),
            start_time: None,
            event_sender,
            event_receiver,
        }
    }

    pub async fn start(&mut self) -> IcnResult<()> {
        info!("Starting network on {}", self.local_addr);
        self.start_time = Some(Instant::now());
        // Implement actual network initialization here
        Ok(())
    }

    pub async fn stop(&mut self) -> IcnResult<()> {
        info!("Stopping network");
        self.start_time = None;
        // Implement network shutdown here
        Ok(())
    }

    pub fn get_connected_peers(&self) -> Vec<SocketAddr> {
        self.peers.keys().cloned().collect()
    }

    pub fn get_uptime(&self) -> Duration {
        self.start_time.map_or(Duration::from_secs(0), |start| start.elapsed())
    }

    pub async fn connect_to_peer(&mut self, peer_addr: SocketAddr) -> IcnResult<()> {
        // Implement peer connection logic
        self.peers.insert(peer_addr, PeerInfo { last_seen: Instant::now() });
        self.event_sender.send(NetworkEvent::PeerConnected(peer_addr)).await
            .map_err(|e| IcnError::Network(format!("Failed to send peer connected event: {}", e)))?;
        Ok(())
    }

    pub async fn disconnect_from_peer(&mut self, peer_addr: &SocketAddr) -> IcnResult<()> {
        // Implement peer disconnection logic
        self.peers.remove(peer_addr);
        self.event_sender.send(NetworkEvent::PeerDisconnected(*peer_addr)).await
            .map_err(|e| IcnError::Network(format!("Failed to send peer disconnected event: {}", e)))?;
        Ok(())
    }

    pub async fn broadcast_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        // Implement transaction broadcasting to all peers
        for peer_addr in self.peers.keys() {
            info!("Broadcasting transaction to peer: {}", peer_addr);
        }
        self.event_sender.send(NetworkEvent::NewTransaction(transaction)).await
            .map_err(|e| IcnError::Network(format!("Failed to send new transaction event: {}", e)))?;
        Ok(())
    }

    pub async fn broadcast_block(&self, block: Block) -> IcnResult<()> {
        // Implement block broadcasting to all peers
        for peer_addr in self.peers.keys() {
            info!("Broadcasting block to peer: {}", peer_addr);
        }
        self.event_sender.send(NetworkEvent::NewBlock(block)).await
            .map_err(|e| IcnError::Network(format!("Failed to send new block event: {}", e)))?;
        Ok(())
    }

    pub async fn receive_event(&mut self) -> Option<NetworkEvent> {
        self.event_receiver.recv().await
    }

    pub async fn handle_incoming_message(&mut self, peer_addr: SocketAddr, message: &[u8]) -> IcnResult<()> {
        info!("Received message from peer {}: {:?}", peer_addr, message);
        Ok(())
    }

    pub async fn broadcast_cross_shard_transaction(&self, cross_shard_tx: CrossShardTransaction) -> IcnResult<()> {
        for peer_addr in self.peers.keys() {
            info!("Broadcasting cross-shard transaction to peer: {}", peer_addr);
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CrossShardTransaction {
    pub transaction: Transaction,
    pub from_shard: u64,
    pub to_shard: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use icn_common::CurrencyType;

    #[test]
    fn test_network_operations() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut network = Network::new("127.0.0.1:8000".parse().unwrap());

            assert!(network.start().await.is_ok());

            let peer_addr: SocketAddr = "127.0.0.1:8001".parse().unwrap();
            assert!(network.connect_to_peer(peer_addr).await.is_ok());
            assert_eq!(network.get_connected_peers().len(), 1);

            assert!(network.disconnect_from_peer(&peer_addr).await.is_ok());
            assert_eq!(network.get_connected_peers().len(), 0);

            let transaction = Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 100.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: chrono::Utc::now().timestamp(),
                signature: None,
            };
            assert!(network.broadcast_transaction(transaction).await.is_ok());

            let block = Block {
                index: 1,
                timestamp: chrono::Utc::now().timestamp(),
                transactions: vec![],
                previous_hash: "previous_hash".to_string(),
                hash: "current_hash".to_string(),
            };
            assert!(network.broadcast_block(block).await.is_ok());

            if let Some(event) = network.receive_event().await {
                match event {
                    NetworkEvent::NewTransaction(_) => println!("Received new transaction event"),
                    NetworkEvent::NewBlock(_) => println!("Received new block event"),
                    NetworkEvent::PeerConnected(_) => println!("Received peer connected event"),
                    NetworkEvent::PeerDisconnected(_) => println!("Received peer disconnected event"),
                }
            }

            assert!(network.stop().await.is_ok());
        });
    }
}
