use icn_common::{IcnResult, IcnError, Transaction, NetworkStats};
use icn_blockchain::Block;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{info, warn, error};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Transaction(Transaction),
    Block(Block),
    PeerConnect(SocketAddr),
    PeerDisconnect(SocketAddr),
}

struct PeerInfo {
    last_seen: Instant,
}

pub struct NetworkManager {
    local_addr: SocketAddr,
    peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
    event_sender: mpsc::Sender<NetworkMessage>,
    event_receiver: mpsc::Receiver<NetworkMessage>,
    start_time: Option<Instant>,
}

impl NetworkManager {
    pub fn new(local_addr: SocketAddr) -> Self {
        let (event_sender, event_receiver) = mpsc::channel(100); // Adjust buffer size as needed
        NetworkManager {
            local_addr,
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver,
            start_time: None,
        }
    }

    pub async fn start(&mut self) -> IcnResult<()> {
        info!("Starting network on {}", self.local_addr);
        self.start_time = Some(Instant::now());

        let listener = TcpListener::bind(self.local_addr).await
            .map_err(|e| IcnError::Network(format!("Failed to bind to address: {}", e)))?;

        let peers = Arc::clone(&self.peers);
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let peer_tx = event_sender.clone();
                let peer_peers = Arc::clone(&peers);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, addr, peer_tx, peer_peers).await {
                        error!("Error handling connection from {}: {}", addr, e);
                    }
                });
            }
        });

        info!("Network started successfully");
        Ok(())
    }

    pub async fn stop(&mut self) -> IcnResult<()> {
        info!("Stopping network");
        self.start_time = None;
        Ok(())
    }

    pub fn get_connected_peers(&self) -> Vec<SocketAddr> {
        self.peers.read().unwrap().keys().cloned().collect()
    }

    pub fn get_uptime(&self) -> Duration {
        self.start_time.map_or(Duration::from_secs(0), |start| start.elapsed())
    }

    pub async fn connect_to_peer(&mut self, peer_addr: SocketAddr) -> IcnResult<()> {
        if self.peers.read().unwrap().contains_key(&peer_addr) {
            return Ok(());  // Already connected
        }

        let stream = TcpStream::connect(peer_addr).await
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer {}: {}", peer_addr, e)))?;

        let peers = Arc::clone(&self.peers);
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer_addr, event_sender, peers).await {
                error!("Error handling connection to {}: {}", peer_addr, e);
            }
        });

        self.peers.write().unwrap().insert(peer_addr, PeerInfo { last_seen: Instant::now() });
        self.event_sender.send(NetworkMessage::PeerConnect(peer_addr)).await
            .map_err(|e| IcnError::Network(format!("Failed to send peer connected event: {}", e)))?;

        info!("Connected to peer: {}", peer_addr);
        Ok(())
    }

    pub async fn disconnect_from_peer(&mut self, peer_addr: &SocketAddr) -> IcnResult<()> {
        self.peers.write().unwrap().remove(peer_addr);
        self.event_sender.send(NetworkMessage::PeerDisconnect(*peer_addr)).await
            .map_err(|e| IcnError::Network(format!("Failed to send peer disconnected event: {}", e)))?;
        Ok(())
    }

    pub async fn broadcast_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        let message = NetworkMessage::Transaction(transaction);
        self.broadcast_message(message).await
    }

    pub async fn broadcast_block(&self, block: Block) -> IcnResult<()> {
        let message = NetworkMessage::Block(block);
        self.broadcast_message(message).await
    }

    async fn broadcast_message(&self, message: NetworkMessage) -> IcnResult<()> {
        let peers = self.peers.read().unwrap();
        for peer_addr in peers.keys() {
            if let Err(e) = self.send_message_to_peer(*peer_addr, message.clone()).await {
                warn!("Failed to send message to peer {}: {}", peer_addr, e);
            }
        }
        Ok(())
    }

    async fn send_message_to_peer(&self, peer_addr: SocketAddr, message: NetworkMessage) -> IcnResult<()> {
        let mut stream = TcpStream::connect(peer_addr).await
            .map_err(|e| IcnError::Network(format!("Failed to connect to peer {}: {}", peer_addr, e)))?;

        let serialized_message = bincode::serialize(&message)
            .map_err(|e| IcnError::Network(format!("Failed to serialize message: {}", e)))?;

        stream.write_all(&serialized_message).await
            .map_err(|e| IcnError::Network(format!("Failed to send message to peer {}: {}", peer_addr, e)))?;

        Ok(())
    }

    pub async fn receive_event(&mut self) -> Option<NetworkMessage> {
        self.event_receiver.recv().await
    }

    pub async fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            node_count: self.peers.read().unwrap().len(),
            total_transactions: 0, // Implement tracking logic
            active_proposals: 0,   // Implement tracking logic
        }
    }
}

#[derive(Clone, Debug)]
pub struct CrossShardTransaction {
    pub transaction: Transaction,
    pub from_shard: u64,
    pub to_shard: u64,
}

async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    event_sender: mpsc::Sender<NetworkMessage>,
    peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
) -> IcnResult<()> {
    let (mut reader, mut writer) = stream.split();
    let mut buffer = vec![0; 1024]; // Use a fixed-size buffer

    loop {
        let bytes_read = reader.read(&mut buffer).await
            .map_err(|e| IcnError::Network(format!("Failed to read from stream: {}", e)))?;

        if bytes_read == 0 {
            // Connection closed
            peers.write().unwrap().remove(&addr);
            event_sender.send(NetworkMessage::PeerDisconnect(addr)).await
                .map_err(|e| IcnError::Network(format!("Failed to send peer disconnected event: {}", e)))?;
            break;
        }

        let message: NetworkMessage = bincode::deserialize(&buffer[..bytes_read])
            .map_err(|e| IcnError::Network(format!("Failed to deserialize message: {}", e)))?;

        event_sender.send(message).await
            .map_err(|e| IcnError::Network(format!("Failed to send message to main thread: {}", e)))?;

        buffer.clear();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_network_operations() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut manager1 = NetworkManager::new("127.0.0.1:8000".parse().unwrap());
            let mut manager2 = NetworkManager::new("127.0.0.1:8001".parse().unwrap());

            manager1.start().await.unwrap();
            manager2.start().await.unwrap();

            manager1.connect_to_peer("127.0.0.1:8001".parse().unwrap()).await.unwrap();

            // Wait a bit for the connection to be established
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            assert_eq!(manager1.get_connected_peers().len(), 1);
            assert_eq!(manager2.get_connected_peers().len(), 1);

            let transaction = Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 100.0,
                currency_type: icn_common::CurrencyType::BasicNeeds,
                timestamp: chrono::Utc::now().timestamp(),
                signature: None,
            };

            manager1.broadcast_transaction(transaction.clone()).await.unwrap();

            // Wait a bit for the message to be processed
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            if let Some(NetworkMessage::Transaction(received_tx)) = manager2.receive_event().await {
                assert_eq!(received_tx, transaction);
            } else {
                panic!("Did not receive expected transaction");
            }

            assert!(manager1.stop().await.is_ok());
        });
    }

    #[test]
    fn test_multiple_peers() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let addr1: SocketAddr = "127.0.0.1:8002".parse().unwrap();
            let addr2: SocketAddr = "127.0.0.1:8003".parse().unwrap();
            let addr3: SocketAddr = "127.0.0.1:8004".parse().unwrap();

            let mut manager1 = NetworkManager::new(addr1);
            let mut manager2 = NetworkManager::new(addr2);
            let mut manager3 = NetworkManager::new(addr3);

            manager1.start().await.unwrap();
            manager2.start().await.unwrap();
            manager3.start().await.unwrap();

            manager1.connect_to_peer(addr2).await.unwrap();
            manager1.connect_to_peer(addr3).await.unwrap();

            // Wait a bit for the connections to be established
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            assert_eq!(manager1.get_connected_peers().len(), 2);
            assert_eq!(manager2.get_connected_peers().len(), 1);
            assert_eq!(manager3.get_connected_peers().len(), 1);

            let transaction = Transaction {
                from: "Charlie".to_string(),
                to: "David".to_string(),
                amount: 200.0,
                currency_type: icn_common::CurrencyType::Education,
                timestamp: chrono::Utc::now().timestamp(),
                signature: None,
            };

            manager1.broadcast_transaction(transaction.clone()).await.unwrap();

            // Wait a bit for the message to be processed
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Check if both manager2 and manager3 received the transaction
            let received2 = manager2.receive_event().await;
            let received3 = manager3.receive_event().await;

            assert!(matches!(received2, Some(NetworkMessage::Transaction(_))));
            assert!(matches!(received3, Some(NetworkMessage::Transaction(_))));
        });
    }

    #[test]
    fn test_peer_disconnect() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let addr1: SocketAddr = "127.0.0.1:8005".parse().unwrap();
            let addr2: SocketAddr = "127.0.0.1:8006".parse().unwrap();

            let mut manager1 = NetworkManager::new(addr1);
            let mut manager2 = NetworkManager::new(addr2);

            manager1.start().await.unwrap();
            manager2.start().await.unwrap();

            manager1.connect_to_peer(addr2).await.unwrap();

            // Wait a bit for the connection to be established
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            assert_eq!(manager1.get_connected_peers().len(), 1);
            assert_eq!(manager2.get_connected_peers().len(), 1);

            // Simulate manager2 disconnecting
            drop(manager2);

            // Wait a bit for the disconnection to be detected
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            assert_eq!(manager1.get_connected_peers().len(), 0);

            // Check if manager1 received a peer disconnect message
            let received = manager1.receive_event().await;
            assert!(matches!(received, Some(NetworkMessage::PeerDisconnect(_))));
        });
    }
}
