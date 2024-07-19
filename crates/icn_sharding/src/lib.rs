use std::collections::HashMap;
use sha2::{Sha256, Digest};
use crate::blockchain::{Block, Transaction};
use crate::network::Node;
use crate::currency::CurrencyType;
use std::sync::{Arc, Mutex};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use log::{info, error, warn, debug};
use crate::error::{Error, Result};
use thiserror::Error;

pub mod cross_shard_communication;

#[derive(Error, Debug)]
pub enum ShardingError {
    #[error("Shard not found: {0}")]
    ShardNotFound(u64),
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("Failed to lock shard: {0}")]
    ShardLockFailed(String),
    #[error("Cross-shard communication error: {0}")]
    CrossShardCommunicationError(String),
}

pub struct Shard {
    pub id: u64,
    pub nodes: Vec<Node>,
    pub blockchain: Vec<Block>,
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
    pub locked_funds: HashMap<String, HashMap<CurrencyType, f64>>,
}

pub struct ShardingManager {
    shards: HashMap<u64, Arc<Mutex<Shard>>>,
    shard_count: u64,
    nodes_per_shard: usize,
    address_to_shard: HashMap<String, u64>,
    current_shard_id: u64,
}

impl ShardingManager {
    pub fn new(shard_count: u64, nodes_per_shard: usize) -> Self {
        let mut shards = HashMap::new();
        for i in 0..shard_count {
            shards.insert(i, Arc::new(Mutex::new(Shard {
                id: i,
                nodes: Vec::new(),
                blockchain: Vec::new(),
                balances: HashMap::new(),
                locked_funds: HashMap::new(),
            })));
        }
        
        info!("Created new ShardingManager with {} shards and {} nodes per shard", shard_count, nodes_per_shard);
        ShardingManager {
            shards,
            shard_count,
            nodes_per_shard,
            address_to_shard: HashMap::new(),
            current_shard_id: 0,
        }
    }

    pub fn get_shard_count(&self) -> u64 {
        self.shard_count
    }

    pub fn get_shard(&self, shard_id: u64) -> Result<Arc<Mutex<Shard>>> {
        self.shards.get(&shard_id)
            .cloned()
            .ok_or_else(|| Error::ShardingError(ShardingError::ShardNotFound(shard_id).to_string()))
    }

    pub fn get_shard_mut(&mut self, shard_id: u64) -> Result<&mut Arc<Mutex<Shard>>> {
        self.shards.get_mut(&shard_id)
            .ok_or_else(|| Error::ShardingError(ShardingError::ShardNotFound(shard_id).to_string()))
    }

    pub fn process_transaction(&self, shard_id: u64, transaction: &Transaction) -> Result<()> {
        let shard = self.get_shard(shard_id)?;
        let mut shard = shard.lock().map_err(|e| Error::ShardingError(ShardingError::ShardLockFailed(e.to_string()).to_string()))?;

        if !self.verify_transaction(&shard, transaction) {
            return Err(Error::ShardingError(ShardingError::InvalidTransaction("Transaction verification failed".to_string()).to_string()));
        }

        self.update_balances(&mut shard, transaction)
    }

    fn update_balances(&self, shard: &mut Shard, transaction: &Transaction) -> Result<()> {
        let sender_balances = shard.balances.entry(transaction.from.clone()).or_insert_with(HashMap::new);
        let sender_balance = sender_balances.entry(transaction.currency_type.clone()).or_insert(0.0);
        
        if *sender_balance < transaction.amount {
            return Err(Error::ShardingError(ShardingError::InsufficientBalance(format!("Insufficient balance for sender: {}", transaction.from)).to_string()));
        }
        
        *sender_balance -= transaction.amount;

        let recipient_balances = shard.balances.entry(transaction.to.clone()).or_insert_with(HashMap::new);
        let recipient_balance = recipient_balances.entry(transaction.currency_type.clone()).or_insert(0.0);
        *recipient_balance += transaction.amount;

        Ok(())
    }

    pub fn transfer_between_shards(&self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> Result<()> {
        let from_shard_arc = self.get_shard(from_shard)?;
        let to_shard_arc = self.get_shard(to_shard)?;
        
        let mut from_shard = from_shard_arc.lock().map_err(|e| Error::ShardingError(ShardingError::ShardLockFailed(e.to_string()).to_string()))?;
        let mut to_shard = to_shard_arc.lock().map_err(|e| Error::ShardingError(ShardingError::ShardLockFailed(e.to_string()).to_string()))?;

        if !self.verify_transaction(&from_shard, transaction) {
            return Err(Error::ShardingError(ShardingError::InvalidTransaction("Transaction verification failed in the source shard".to_string()).to_string()));
        }

        self.lock_funds(&mut from_shard, &transaction.from, &transaction.currency_type, transaction.amount)?;
        self.add_balance_to_shard(&mut to_shard, &transaction.to, &transaction.currency_type, transaction.amount)?;

        info!("Cross-shard transaction prepared from shard {} to shard {}", from_shard.id, to_shard.id);
        Ok(())
    }

    pub fn lock_funds(&self, shard: &mut Shard, address: &str, currency_type: &CurrencyType, amount: f64) -> Result<()> {
        let sender_balances = shard.balances.get_mut(address)
            .ok_or_else(|| Error::ShardingError(ShardingError::InsufficientBalance("Sender not found".to_string()).to_string()))?;
        
        let balance = sender_balances.get_mut(currency_type)
            .ok_or_else(|| Error::ShardingError(ShardingError::InsufficientBalance("Currency not found".to_string()).to_string()))?;

        if *balance < amount {
            return Err(Error::ShardingError(ShardingError::InsufficientBalance("Insufficient balance".to_string()).to_string()));
        }

        *balance -= amount;

        shard.locked_funds
            .entry(address.to_string())
            .or_insert_with(HashMap::new)
            .entry(currency_type.clone())
            .and_modify(|e| *e += amount)
            .or_insert(amount);

        Ok(())
    }

    pub fn commit_cross_shard_transaction(&self, transaction: &Transaction, from_shard: u64, to_shard: u64) -> Result<()> {
        let from_shard_arc = self.get_shard(from_shard)?;
        let to_shard_arc = self.get_shard(to_shard)?;
        
        let mut from_shard = from_shard_arc.lock().map_err(|e| Error::ShardingError(ShardingError::ShardLockFailed(e.to_string()).to_string()))?;
        let mut to_shard = to_shard_arc.lock().map_err(|e| Error::ShardingError(ShardingError::ShardLockFailed(e.to_string()).to_string()))?;

        self.remove_fund_lock(&mut from_shard, &transaction.from, &transaction.currency_type, transaction.amount)?;
        self.finalize_balance_update(&mut to_shard, &transaction.to, &transaction.currency_type, transaction.amount)?;

        info!("Cross-shard transaction committed from shard {} to shard {}", from_shard.id, to_shard.id);
        Ok(())
    }

    fn remove_fund_lock(&self, shard: &mut Shard, address: &str, currency_type: &CurrencyType, amount: f64) -> Result<()> {
        let locked_funds = shard.locked_funds.get_mut(address)
            .ok_or_else(|| Error::ShardingError(ShardingError::InsufficientBalance("No locked funds found".to_string()).to_string()))?;

        let locked_amount = locked_funds.get_mut(currency_type)
            .ok_or_else(|| Error::ShardingError(ShardingError::InsufficientBalance("No locked funds for this currency".to_string()).to_string()))?;

        if *locked_amount < amount {
            return Err(Error::ShardingError(ShardingError::InsufficientBalance("Insufficient locked funds".to_string()).to_string()));
        }

        *locked_amount -= amount;

        if *locked_amount == 0.0 {
            locked_funds.remove(currency_type);
        }

        if locked_funds.is_empty() {
            shard.locked_funds.remove(address);
        }

        Ok(())
    }

    fn finalize_balance_update(&self, shard: &mut Shard, address: &str, currency_type: &CurrencyType, amount: f64) -> Result<()> {
        let balances = shard.balances.entry(address.to_string()).or_insert_with(HashMap::new);
        let balance = balances.entry(currency_type.clone()).or_insert(0.0);
        *balance += amount;
        Ok(())
    }

    pub fn add_balance(&self, address: &str, currency_type: CurrencyType, amount: f64) -> Result<()> {
        let shard_id = self.get_shard_for_address(address);
        let shard = self.get_shard(shard_id)?;
        let mut shard = shard.lock().map_err(|e| Error::ShardingError(ShardingError::ShardLockFailed(e.to_string()).to_string()))?;
    
        let balance = shard.balances
            .entry(address.to_string())
            .or_insert_with(HashMap::new)
            .entry(currency_type.clone())
            .or_insert(0.0);
        *balance += amount;
        
        info!("Added balance of {} {} for address {} in shard {}", amount, currency_type, address, shard_id);
        Ok(())
    }

    fn add_balance_to_shard(&self, shard: &mut Shard, address: &str, currency_type: &CurrencyType, amount: f64) -> Result<()> {
        let balance = shard.balances
            .entry(address.to_string())
            .or_insert_with(HashMap::new)
            .entry(currency_type.clone())
            .or_insert(0.0);
        *balance += amount;
        Ok(())
    }

    pub fn assign_node_to_shard(&self, node: Node, shard_id: u64) -> Result<()> {
        let shard_arc = self.get_shard(shard_id)?;
        let mut shard = shard_arc.lock().map_err(|e| Error::ShardingError(ShardingError::ShardLockFailed(e.to_string()).to_string()))?;
        if shard.nodes.len() >= self.nodes_per_shard {
            error!("Failed to assign node to shard {}: Shard is full", shard_id);
            return Err(Error::ShardingError(ShardingError::ShardNotFound(shard_id).to_string()));
        }
        shard.nodes.push(node.clone());
        info!("Assigned node {} to shard {}", node.id, shard_id);
        Ok(())
    }

    pub fn get_shard_for_data(&self, data: &[u8]) -> u64 {
        let hash = self.hash_data(data);
        hash % self.shard_count
    }

    pub fn get_shard_for_address(&self, address: &str) -> u64 {
        *self.address_to_shard.get(address).unwrap_or(&(self.hash_data(address.as_bytes()) % self.shard_count))
    }

    pub fn get_current_shard_id(&self) -> u64 {
        self.current_shard_id
    }

    pub fn set_current_shard_id(&mut self, shard_id: u64) {
        self.current_shard_id = shard_id;
    }

    pub fn add_address_to_shard(&mut self, address: String, shard_id: u64) {
        self.address_to_shard.insert(address.clone(), shard_id);
        info!("Added address {} to shard {}", address, shard_id);
    }

    pub fn initialize_balance(&self, address: String, currency_type: CurrencyType, amount: f64) -> Result<()> {
        let shard_id = self.get_shard_for_address(&address);
        let shard = self.get_shard(shard_id)?;
        let mut shard = shard.lock().map_err(|e| Error::ShardingError(ShardingError::ShardLockFailed(e.to_string()).to_string()))?;
        
        shard.balances
            .entry(address.clone())
            .or_insert_with(HashMap::new)
            .insert(currency_type.clone(), amount);
        
        info!("Initialized balance of {} {} for {} in shard {}", amount, currency_type, address, shard_id);
        Ok(())
    }

    pub fn get_balance(&self, address: String, currency_type: CurrencyType) -> Result<f64> {
        let shard_id = self.get_shard_for_address(&address);
        let shard = self.get_shard(shard_id)?;
        let shard = shard.lock().map_err(|e| Error::ShardingError(ShardingError::ShardLockFailed(e.to_string()).to_string()))?;
        
        let balance = shard.balances
            .get(&address)
            .and_then(|balances| balances.get(&currency_type))
            .cloned()
            .unwrap_or(0.0);
        
        Ok(balance)
    }

    pub fn lock_funds_in_shard(&self, shard_id: u64, address: &str, currency_type: &CurrencyType, amount: f64) -> Result<()> {
        let shard = self.get_shard(shard_id)?;
        let mut shard = shard.lock().map_err(|e| Error::ShardingError(ShardingError::ShardLockFailed(e.to_string()).to_string()))?;
        self.lock_funds(&mut shard, address, currency_type, amount)
    }

    fn verify_transaction(&self, shard: &Shard, transaction: &Transaction) -> bool {
        debug!("Checking balance for sender: {}", transaction.from);
        if let Some(sender_balances) = shard.balances.get(&transaction.from) {
            if let Some(balance) = sender_balances.get(&transaction.currency_type) {
                if *balance < transaction.amount {
                    warn!("Insufficient balance for sender: {}", transaction.from);
                    return false;
                }
            } else {
                warn!("Sender does not have the required currency type");
                return false;
            }
        } else {
            warn!("Sender not found in this shard");
            return false;
        }

        debug!("Verifying transaction signature");
        if let (Some(public_key), Some(signature)) = (&transaction.public_key, &transaction.signature) {
            let public_key = PublicKey::from_bytes(public_key).unwrap();
            let signature = Signature::from_bytes(signature).unwrap();
            let message = transaction.to_bytes();
            if public_key.verify(&message, &signature).is_err() {
                warn!("Signature verification failed");
                return false;
            }
        } else {
            warn!("Missing public key or signature");
            return false;
        }

        true
    }

    fn hash_data(&self, data: &[u8]) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let hash_bytes: [u8; 8] = result[..8].try_into().unwrap_or([0; 8]);
        u64::from_le_bytes(hash_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Node;
    use crate::network::node::NodeType;
    use crate::currency::CurrencyType;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_create_sharding_manager() {
        let manager = ShardingManager::new(4, 10);
        assert_eq!(manager.shards.len(), 4);
        assert_eq!(manager.shard_count, 4);
        assert_eq!(manager.nodes_per_shard, 10);
    }

    #[test]
    fn test_assign_node_to_shard() {
        let manager = ShardingManager::new(4, 2);
        let node1 = Node::new("node1", NodeType::PersonalDevice, "127.0.0.1:8000");
        let node2 = Node::new("node2", NodeType::PersonalDevice, "127.0.0.1:8001");

        assert!(manager.assign_node_to_shard(node1.clone(), 0).is_ok());
        assert!(manager.assign_node_to_shard(node2.clone(), 1).is_ok());

        assert_eq!(manager.get_shard(0).unwrap().lock().unwrap().nodes.len(), 1);
        assert_eq!(manager.get_shard(1).unwrap().lock().unwrap().nodes.len(), 1);
    }

    #[test]
    fn test_process_transaction() {
        let mut manager = ShardingManager::new(4, 10);
        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        manager.add_address_to_shard("Alice".to_string(), 0);
        manager.add_address_to_shard("Bob".to_string(), 0);
        manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0).unwrap();

        assert!(manager.process_transaction(0, &transaction).is_ok());

        assert_eq!(manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds).unwrap(), 900.0);
        assert_eq!(manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds).unwrap(), 100.0);
    }

    #[test]
    fn test_cross_shard_transaction() {
        let mut manager = ShardingManager::new(4, 10);
        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        manager.add_address_to_shard("Alice".to_string(), 0);
        manager.add_address_to_shard("Bob".to_string(), 1);
        manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0).unwrap();

        assert!(manager.transfer_between_shards(0, 1, &transaction).is_ok());
        assert!(manager.commit_cross_shard_transaction(&transaction, 0, 1).is_ok());

        assert_eq!(manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds).unwrap(), 900.0);
        assert_eq!(manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds).unwrap(), 100.0);
    }

    #[test]
    fn test_get_shard_for_data() {
        let manager = ShardingManager::new(4, 10);
        let data1 = b"test_data_1";
        let data2 = b"test_data_2";

        let shard1 = manager.get_shard_for_data(data1);
        let shard2 = manager.get_shard_for_data(data2);

        assert!(shard1 < 4);
        assert!(shard2 < 4);
        // There's a small chance these could be equal, but it's unlikely
        assert_ne!(shard1, shard2);
    }

    #[test]
    fn test_get_shard_for_address() {
        let mut manager = ShardingManager::new(4, 10);
        manager.add_address_to_shard("Alice".to_string(), 2);

        assert_eq!(manager.get_shard_for_address("Alice"), 2);
        
        let bob_shard = manager.get_shard_for_address("Bob");
        assert!(bob_shard < 4);
    }

    #[test]
    fn test_add_and_get_balance() {
        let mut manager = ShardingManager::new(4, 10);
        manager.add_address_to_shard("Charlie".to_string(), 3);

        assert!(manager.add_balance("Charlie", CurrencyType::BasicNeeds, 500.0).is_ok());
        assert_eq!(manager.get_balance("Charlie".to_string(), CurrencyType::BasicNeeds).unwrap(), 500.0);

        assert!(manager.add_balance("Charlie", CurrencyType::BasicNeeds, 250.0).is_ok());
        assert_eq!(manager.get_balance("Charlie".to_string(), CurrencyType::BasicNeeds).unwrap(), 750.0);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut manager = ShardingManager::new(4, 10);
        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "David".to_string(),
            "Eve".to_string(),
            1000.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        manager.add_address_to_shard("David".to_string(), 0);
        manager.add_address_to_shard("Eve".to_string(), 0);
        manager.initialize_balance("David".to_string(), CurrencyType::BasicNeeds, 500.0).unwrap();

        assert!(manager.process_transaction(0, &transaction).is_err());
        assert_eq!(manager.get_balance("David".to_string(), CurrencyType::BasicNeeds).unwrap(), 500.0);
        assert_eq!(manager.get_balance("Eve".to_string(), CurrencyType::BasicNeeds).unwrap(), 0.0);
    }

    #[test]
    fn test_cross_shard_insufficient_balance() {
        let mut manager = ShardingManager::new(4, 10);
        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Frank".to_string(),
            "Grace".to_string(),
            1000.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        manager.add_address_to_shard("Frank".to_string(), 0);
        manager.add_address_to_shard("Grace".to_string(), 1);
        manager.initialize_balance("Frank".to_string(), CurrencyType::BasicNeeds, 500.0).unwrap();

        assert!(manager.transfer_between_shards(0, 1, &transaction).is_err());
        assert_eq!(manager.get_balance("Frank".to_string(), CurrencyType::BasicNeeds).unwrap(), 500.0);
        assert_eq!(manager.get_balance("Grace".to_string(), CurrencyType::BasicNeeds).unwrap(), 0.0);
    }
}