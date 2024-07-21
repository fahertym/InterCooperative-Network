use icn_common::{IcnResult, IcnError, Block, Transaction, CurrencyType};
use std::collections::{HashMap, HashSet};
use sha2::{Sha256, Digest};
use chrono::Utc;

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
        let from_shard = self.get_shard_for_address(&transaction.from);
        let to_shard = self.get_shard_for_address(&transaction.to);

        if from_shard == to_shard {
            self.process_intra_shard_transaction(from_shard, transaction)
        } else {
            self.process_cross_shard_transaction(from_shard, to_shard, transaction)
        }
    }

    fn process_intra_shard_transaction(&mut self, shard_id: u64, transaction: &Transaction) -> IcnResult<()> {
        let shard = self.shards.get_mut(&shard_id).ok_or_else(|| IcnError::Sharding("Shard not found".to_string()))?;

        if !self.verify_transaction(shard, transaction) {
            return Err(IcnError::Sharding("Invalid transaction".to_string()));
        }

        shard.pending_transactions.push(transaction.clone());
        self.update_balances(shard, transaction)
    }

    fn process_cross_shard_transaction(&mut self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> IcnResult<()> {
        // Lock funds in the source shard
        {
            let from_shard = self.shards.get_mut(&from_shard).ok_or_else(|| IcnError::Sharding("Source shard not found".to_string()))?;
            if !self.verify_transaction(from_shard, transaction) {
                return Err(IcnError::Sharding("Invalid transaction".to_string()));
            }
            self.update_balances(from_shard, transaction)?;
        }

        // Update recipient's balance in the destination shard
        {
            let to_shard = self.shards.get_mut(&to_shard).ok_or_else(|| IcnError::Sharding("Destination shard not found".to_string()))?;
            let mut reverse_transaction = transaction.clone();
            reverse_transaction.from = transaction.to.clone();
            reverse_transaction.to = transaction.from.clone();
            self.update_balances(to_shard, &reverse_transaction)?;
        }

        Ok(())
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
            timestamp: Utc::now().timestamp(),
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

    pub fn rebalance_shards(&mut self) -> IcnResult<()> {
        let total_addresses: usize = self.address_to_shard.len();
        let target_addresses_per_shard = total_addresses / self.shard_count as usize;
        let mut overloaded_shards: Vec<u64> = Vec::new();
        let mut underloaded_shards: Vec<u64> = Vec::new();

        // Identify overloaded and underloaded shards
        for shard_id in 0..self.shard_count {
            let shard_addresses: usize = self.address_to_shard.values().filter(|&&s| s == shard_id).count();
            if shard_addresses > target_addresses_per_shard + 1 {
                overloaded_shards.push(shard_id);
            } else if shard_addresses < target_addresses_per_shard {
                underloaded_shards.push(shard_id);
            }
        }

        // Rebalance addresses between shards
        for overloaded_shard in overloaded_shards {
            let addresses_to_move: Vec<String> = self.address_to_shard
                .iter()
                .filter(|&(_, &shard)| shard == overloaded_shard)
                .map(|(addr, _)| addr.clone())
                .take((self.address_to_shard.values().filter(|&&s| s == overloaded_shard).count() - target_addresses_per_shard) as usize)
                .collect();

            for address in addresses_to_move {
                if let Some(target_shard) = underloaded_shards.first() {
                    self.move_address_to_shard(&address, *target_shard)?;
                    if self.address_to_shard.values().filter(|&&s| s == *target_shard).count() >= target_addresses_per_shard {
                        underloaded_shards.remove(0);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn handle_shard_failure(&mut self, failed_shard_id: u64) -> IcnResult<()> {
        if failed_shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".to_string()));
        }

        // Collect addresses from the failed shard
        let failed_shard_addresses: Vec<String> = self.address_to_shard
            .iter()
            .filter(|&(_, &shard)| shard == failed_shard_id)
            .map(|(addr, _)| addr.clone())
            .collect();

        // Redistribute addresses to other shards
        let mut target_shard_id = (failed_shard_id + 1) % self.shard_count;
        for address in failed_shard_addresses {
            while target_shard_id == failed_shard_id {
                target_shard_id = (target_shard_id + 1) % self.shard_count;
            }
            self.move_address_to_shard(&address, target_shard_id)?;
            target_shard_id = (target_shard_id + 1) % self.shard_count;
        }

        // Clear the failed shard's data
        if let Some(failed_shard) = self.shards.get_mut(&failed_shard_id) {
            failed_shard.blockchain = vec![Block::genesis()];
            failed_shard.balances.clear();
            failed_shard.pending_transactions.clear();
        }

        // Trigger a network-wide state update
        self.update_network_state()?;

        Ok(())
    }

    fn move_address_to_shard(&mut self, address: &str, new_shard_id: u64) -> IcnResult<()> {
        let old_shard_id = self.get_shard_for_address(address);
        if old_shard_id == new_shard_id {
            return Ok(());
        }

        // Move balances
        if let Some(old_shard) = self.shards.get_mut(&old_shard_id) {
            if let Some(balances) = old_shard.balances.remove(address) {
                if let Some(new_shard) = self.shards.get_mut(&new_shard_id) {
                    new_shard.balances.insert(address.to_string(), balances);
                }
            }
        }

        // Update address_to_shard mapping
        self.address_to_shard.insert(address.to_string(), new_shard_id);

        Ok(())
    }

    fn update_network_state(&self) -> IcnResult<()> {
        // In a real implementation, this method would broadcast the updated state
        // to all nodes in the network, ensuring consistency across the system.
        // For now, we'll just log the action.
        println!("Network state update triggered");
        Ok(())
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

        assert!(manager.process_transaction(&cross_shard_tx).is_ok());
        assert_eq!(manager.get_balance("Alice", &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(manager.get_balance("Bob", &CurrencyType::BasicNeeds).unwrap(), 150.0);

        // Test getting shard state
        let shard_state = manager.shards.get(&0).unwrap();
        assert_eq!(shard_state.id, 0);
        assert_eq!(shard_state.blockchain.len(), 2); // Genesis block + 1 new block

        // Test invalid shard access
        assert!(manager.get_balance("NonexistentUser", &CurrencyType::BasicNeeds).is_err());
    }

    #[test]
    fn test_rebalance_shards() {
        let mut manager = ShardingManager::new(3);
        
        // Add addresses to shards unevenly
        for i in 0..10 {
            manager.add_address_to_shard(format!("Address{}", i), 0).unwrap();
        }
        for i in 10..15 {
            manager.add_address_to_shard(format!("Address{}", i), 1).unwrap();
        }
        for i in 15..17 {
            manager.add_address_to_shard(format!("Address{}", i), 2).unwrap();
        }

        // Rebalance shards
        manager.rebalance_shards().unwrap();

        // Check if shards are more evenly balanced
        let shard_0_count = manager.address_to_shard.values().filter(|&&s| s == 0).count();
        let shard_1_count = manager.address_to_shard.values().filter(|&&s| s == 1).count();
        let shard_2_count = manager.address_to_shard.values().filter(|&&s| s == 2).count();

        assert!(shard_0_count >= 5 && shard_0_count <= 6);
        assert!(shard_1_count >= 5 && shard_1_count <= 6);
        assert!(shard_2_count >= 5 && shard_2_count <= 6);
    }

    #[test]
    fn test_handle_shard_failure() {
        let mut manager = ShardingManager::new(3);
        
        // Add addresses and balances to shards
        for i in 0..9 {
            let address = format!("Address{}", i);
            manager.add_address_to_shard(address.clone(), i / 3).unwrap();
            let shard = manager.shards.get_mut(&(i / 3)).unwrap();
            shard.balances.entry(address).or_insert_with(HashMap::new).insert(CurrencyType::BasicNeeds, 100.0);
        }

        // Simulate failure of shard 1
        manager.handle_shard_failure(1).unwrap();

        // Check if addresses from shard 1 have been redistributed
        assert_eq!(manager.address_to_shard.values().filter(|&&s| s == 1).count(), 0);

        // Check if balances have been moved
        for i in 3..6 {
            let address = format!("Address{}", i);
            let new_shard_id = manager.get_shard_for_address(&address);
            assert_ne!(new_shard_id, 1);
            let balance = manager.get_balance(&address, &CurrencyType::BasicNeeds).unwrap();
            assert_eq!(balance, 100.0);
        }

        // Check if shard 1 has been reset
        let shard_1 = manager.shards.get(&1).unwrap();
        assert_eq!(shard_1.blockchain.len(), 1); // Only genesis block
        assert!(shard_1.balances.is_empty());
        assert!(shard_1.pending_transactions.is_empty());
    }
}
