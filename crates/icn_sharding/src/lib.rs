use icn_common_types::{IcnResult, IcnError, Block, Transaction, CurrencyType};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

pub struct Shard {
    pub id: u64,
    pub blockchain: Vec<Block>,
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
    pub pending_transactions: Vec<Transaction>,
}

pub struct ShardingManager {
    shards: HashMap<u64, Shard>,
    shard_count: u64,
    address_to_shard: HashMap<String, u64>,
}

impl ShardingManager {
    pub fn new(shard_count: u64) -> Self {
        let mut shards = HashMap::new();
        for i in 0..shard_count {
            shards.insert(i, Shard {
                id: i,
                blockchain: vec![Block::genesis()],
                balances: HashMap::new(),
                pending_transactions: Vec::new(),
            });
        }
        
        ShardingManager {
            shards,
            shard_count,
            address_to_shard: HashMap::new(),
        }
    }

    pub fn get_shard_count(&self) -> u64 {
        self.shard_count
    }

    pub fn process_transaction(&mut self, transaction: &Transaction) -> IcnResult<()> {
        let shard_id = self.get_shard_for_address(&transaction.from);
        let shard = self.shards.get_mut(&shard_id).ok_or_else(|| IcnError::Sharding("Shard not found".to_string()))?;

        if !self.verify_transaction(shard, transaction) {
            return Err(IcnError::Sharding("Invalid transaction".to_string()));
        }

        shard.pending_transactions.push(transaction.clone());
        self.update_balances(shard, transaction)
    }

    fn update_balances(&mut self, shard: &mut Shard, transaction: &Transaction) -> IcnResult<()> {
        let sender_balances = shard.balances.entry(transaction.from.clone()).or_insert_with(HashMap::new);
        let sender_balance = sender_balances.entry(transaction.currency_type.clone()).or_insert(0.0);
        
        if *sender_balance < transaction.amount {
            return Err(IcnError::Sharding("Insufficient balance".to_string()));
        }
        
        *sender_balance -= transaction.amount;

        let recipient_balances = shard.balances.entry(transaction.to.clone()).or_insert_with(HashMap::new);
        let recipient_balance = recipient_balances.entry(transaction.currency_type.clone()).or_insert(0.0);
        *recipient_balance += transaction.amount;

        Ok(())
    }

    pub fn create_block(&mut self, shard_id: u64) -> IcnResult<Block> {
        let shard = self.shards.get_mut(&shard_id).ok_or_else(|| IcnError::Sharding("Shard not found".to_string()))?;
        let previous_block = shard.blockchain.last().ok_or_else(|| IcnError::Sharding("No previous block found".to_string()))?;
        
        let new_block = Block {
            index: shard.blockchain.len() as u64,
            timestamp: chrono::Utc::now().timestamp(),
            transactions: shard.pending_transactions.clone(),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(), // Will be set later
        };

        let new_block = self.calculate_block_hash(new_block);
        shard.blockchain.push(new_block.clone());
        shard.pending_transactions.clear();

        Ok(new_block)
    }

    pub fn get_shard_for_address(&self, address: &str) -> u64 {
        *self.address_to_shard.get(address).unwrap_or(&(self.hash_address(address) % self.shard_count))
    }

    pub fn add_address_to_shard(&mut self, address: String, shard_id: u64) -> IcnResult<()> {
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".to_string()));
        }
        self.address_to_shard.insert(address, shard_id);
        Ok(())
    }
    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let shard_id = self.get_shard_for_address(address);
        let shard = self.shards.get(&shard_id).ok_or_else(|| IcnError::Sharding("Shard not found".to_string()))?;
        
        Ok(shard.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .cloned()
            .unwrap_or(0.0))
    }

    fn verify_transaction(&self, shard: &Shard, transaction: &Transaction) -> bool {
        if let Some(sender_balances) = shard.balances.get(&transaction.from) {
            if let Some(balance) = sender_balances.get(&transaction.currency_type) {
                return *balance >= transaction.amount;
            }
        }
        false
    }

    fn hash_address(&self, address: &str) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(address.as_bytes());
        let result = hasher.finalize();
        let hash_bytes: [u8; 8] = result[..8].try_into().unwrap_or([0; 8]);
        u64::from_le_bytes(hash_bytes)
    }

    fn calculate_block_hash(&self, mut block: Block) -> Block {
        let mut hasher = Sha256::new();
        hasher.update(block.index.to_string().as_bytes());
        hasher.update(block.timestamp.to_string().as_bytes());
        for transaction in &block.transactions {
            hasher.update(transaction.from.as_bytes());
            hasher.update(transaction.to.as_bytes());
            hasher.update(transaction.amount.to_string().as_bytes());
            hasher.update(format!("{:?}", transaction.currency_type).as_bytes());
        }
        hasher.update(block.previous_hash.as_bytes());
        
        let hash = format!("{:x}", hasher.finalize());
        block.hash = hash;
        block
    }

    pub fn transfer_between_shards(&mut self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> IcnResult<()> {
        // Deduct from the source shard
        let from_shard = self.shards.get_mut(&from_shard).ok_or_else(|| IcnError::Sharding("Source shard not found".to_string()))?;
        self.update_balances(from_shard, transaction)?;

        // Add to the destination shard
        let to_shard = self.shards.get_mut(&to_shard).ok_or_else(|| IcnError::Sharding("Destination shard not found".to_string()))?;
        let mut reverse_transaction = transaction.clone();
        reverse_transaction.from = transaction.to.clone();
        reverse_transaction.to = transaction.from.clone();
        self.update_balances(to_shard, &reverse_transaction)?;

        Ok(())
    }

    pub fn get_shard_state(&self, shard_id: u64) -> IcnResult<&Shard> {
        self.shards.get(&shard_id).ok_or_else(|| IcnError::Sharding("Shard not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharding_manager() {
        let mut manager = ShardingManager::new(4);
        assert_eq!(manager.get_shard_count(), 4);

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        manager.add_address_to_shard("Alice".to_string(), 0).unwrap();
        manager.add_address_to_shard("Bob".to_string(), 1).unwrap();

        // Initialize Alice's balance
        let alice_shard = manager.shards.get_mut(&0).unwrap();
        alice_shard.balances.entry("Alice".to_string()).or_insert_with(HashMap::new).insert(CurrencyType::BasicNeeds, 200.0);

        assert!(manager.process_transaction(&transaction).is_ok());

        assert_eq!(manager.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 100.0);
        assert_eq!(manager.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 100.0);

        // Test block creation
        let block = manager.create_block(0).unwrap();
        assert_eq!(block.index, 1);
        assert_eq!(block.transactions.len(), 1);

        // Test cross-shard transaction
        let cross_shard_tx = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 1,
            signature: None,
        };

        assert!(manager.transfer_between_shards(0, 1, &cross_shard_tx).is_ok());
        assert_eq!(manager.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(manager.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 150.0);

        // Test getting shard state
        let shard_state = manager.get_shard_state(0).unwrap();
        assert_eq!(shard_state.id, 0);
        assert_eq!(shard_state.blockchain.len(), 2); // Genesis block + 1 new block

        // Test invalid shard access
        assert!(manager.get_shard_state(10).is_err());
    }
}