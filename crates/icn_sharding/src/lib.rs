// File: crates/icn_sharding/src/lib.rs

use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, warn, error};

pub struct Shard {
    pub id: u64,
    pub transactions: Vec<Transaction>,
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

pub struct ShardingManager {
    shards: Arc<RwLock<Vec<Shard>>>,
    shard_count: u64,
    address_to_shard: HashMap<String, u64>,
}

impl ShardingManager {
    pub fn new(shard_count: u64) -> Self {
        let mut shards = Vec::new();
        for i in 0..shard_count {
            shards.push(Shard {
                id: i,
                transactions: Vec::new(),
                balances: HashMap::new(),
            });
        }

        ShardingManager {
            shards: Arc::new(RwLock::new(shards)),
            shard_count,
            address_to_shard: HashMap::new(),
        }
    }

    pub fn process_transaction(&self, transaction: &Transaction) -> IcnResult<()> {
        let from_shard = self.get_shard_for_address(&transaction.from);
        let to_shard = self.get_shard_for_address(&transaction.to);

        if from_shard == to_shard {
            self.process_intra_shard_transaction(from_shard, transaction)
        } else {
            self.process_cross_shard_transaction(from_shard, to_shard, transaction)
        }
    }

    fn process_intra_shard_transaction(&self, shard_id: u64, transaction: &Transaction) -> IcnResult<()> {
        let mut shards = self.shards.write().map_err(|_| IcnError::Sharding("Failed to acquire write lock".into()))?;
        let shard = &mut shards[shard_id as usize];

        let from_balance = shard.balances
            .entry(transaction.from.clone())
            .or_default()
            .entry(transaction.currency_type.clone())
            .or_insert(0.0);

        if *from_balance < transaction.amount {
            return Err(IcnError::Sharding("Insufficient balance".into()));
        }

        *from_balance -= transaction.amount;

        let to_balance = shard.balances
            .entry(transaction.to.clone())
            .or_default()
            .entry(transaction.currency_type.clone())
            .or_insert(0.0);

        *to_balance += transaction.amount;

        shard.transactions.push(transaction.clone());
        Ok(())
    }

    fn process_cross_shard_transaction(&self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> IcnResult<()> {
        self.lock_funds(from_shard, &transaction.from, transaction.amount, &transaction.currency_type)?;
        self.transfer_between_shards(from_shard, to_shard, transaction)?;
        Ok(())
    }

    fn lock_funds(&self, shard_id: u64, address: &str, amount: f64, currency_type: &CurrencyType) -> IcnResult<()> {
        let mut shards = self.shards.write().map_err(|_| IcnError::Sharding("Failed to acquire write lock".into()))?;
        let shard = &mut shards[shard_id as usize];

        let balance = shard.balances
            .entry(address.to_string())
            .or_default()
            .entry(currency_type.clone())
            .or_insert(0.0);

        if *balance < amount {
            return Err(IcnError::Sharding("Insufficient balance to lock".into()));
        }

        *balance -= amount;
        Ok(())
    }

    fn transfer_between_shards(&self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> IcnResult<()> {
        let mut shards = self.shards.write().map_err(|_| IcnError::Sharding("Failed to acquire write lock".into()))?;

        let to_shard = &mut shards[to_shard as usize];
        let to_balance = to_shard.balances
            .entry(transaction.to.clone())
            .or_default()
            .entry(transaction.currency_type.clone())
            .or_insert(0.0);

        *to_balance += transaction.amount;

        shards[from_shard as usize].transactions.push(transaction.clone());
        shards[to_shard as usize].transactions.push(transaction.clone());

        Ok(())
    }

    pub fn get_shard_for_address(&self, address: &str) -> u64 {
        *self.address_to_shard.get(address).unwrap_or(&(self.hash_address(address) % self.shard_count))
    }

    fn hash_address(&self, address: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        address.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let shard_id = self.get_shard_for_address(address);
        let shards = self.shards.read().map_err(|_| IcnError::Sharding("Failed to acquire read lock".into()))?;
        let shard = &shards[shard_id as usize];
        
        Ok(shard.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .cloned()
            .unwrap_or(0.0))
    }

    pub fn initialize_balance(&self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let shard_id = self.get_shard_for_address(address);
        let mut shards = self.shards.write().map_err(|_| IcnError::Sharding("Failed to acquire write lock".into()))?;
        let shard = &mut shards[shard_id as usize];

        shard.balances
            .entry(address.to_string())
            .or_default()
            .insert(currency_type.clone(), amount);

        Ok(())
    }

    pub fn get_shard_transactions(&self, shard_id: u64) -> IcnResult<Vec<Transaction>> {
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding(format!("Invalid shard ID: {}", shard_id)));
        }

        let shards = self.shards.read().map_err(|_| IcnError::Sharding("Failed to acquire read lock".into()))?;
        Ok(shards[shard_id as usize].transactions.clone())
    }

    pub fn get_shard_addresses(&self, shard_id: u64) -> IcnResult<Vec<String>> {
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding(format!("Invalid shard ID: {}", shard_id)));
        }

        let shards = self.shards.read().map_err(|_| IcnError::Sharding("Failed to acquire read lock".into()))?;
        Ok(shards[shard_id as usize].balances.keys().cloned().collect())
    }

    pub fn get_shard_currencies(&self, shard_id: u64) -> IcnResult<Vec<CurrencyType>> {
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding(format!("Invalid shard ID: {}", shard_id)));
        }

        let shards = self.shards.read().map_err(|_| IcnError::Sharding("Failed to acquire read lock".into()))?;
        let mut currencies = std::collections::HashSet::new();
        for balances in shards[shard_id as usize].balances.values() {
            currencies.extend(balances.keys().cloned());
        }

        Ok(currencies.into_iter().collect())
    }

    pub fn resize_shards(&mut self, new_shard_count: u64) -> IcnResult<()> {
        if new_shard_count == 0 {
            return Err(IcnError::Sharding("Shard count must be greater than zero".into()));
        }

        let mut new_shards = Vec::with_capacity(new_shard_count as usize);
        for i in 0..new_shard_count {
            new_shards.push(Shard {
                id: i,
                transactions: Vec::new(),
                balances: HashMap::new(),
            });
        }

        let old_shards = std::mem::replace(&mut *self.shards.write().unwrap(), new_shards);

        // Redistribute balances and transactions
        for (old_shard_id, old_shard) in old_shards.into_iter().enumerate() {
            for (address, balances) in old_shard.balances {
                let new_shard_id = self.get_shard_for_address(&address);
                let new_shard = &mut self.shards.write().unwrap()[new_shard_id as usize];
                new_shard.balances.insert(address, balances);
            }

            for transaction in old_shard.transactions {
                let new_shard_id = self.get_shard_for_address(&transaction.from);
                let new_shard = &mut self.shards.write().unwrap()[new_shard_id as usize];
                new_shard.transactions.push(transaction);
            }
        }

        self.shard_count = new_shard_count;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_assignment() {
        let manager = ShardingManager::new(4);
        let address = "0x1234567890123456789012345678901234567890".to_string();
        let shard_id = manager.get_shard_for_address(&address);
        assert!(shard_id < 4);
    }

    #[test]
    fn test_intra_shard_transaction() {
        let manager = ShardingManager::new(4);
        let from_address = "0x1111111111111111111111111111111111111111".to_string();
        let to_address = "0x2222222222222222222222222222222222222222".to_string();

        manager.initialize_balance(&from_address, &CurrencyType::BasicNeeds, 100.0).unwrap();

        let transaction = Transaction {
            from: from_address.clone(),
            to: to_address.clone(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        assert!(manager.process_transaction(&transaction).is_ok());

        assert_eq!(manager.get_balance(&from_address, &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(manager.get_balance(&to_address, &CurrencyType::BasicNeeds).unwrap(), 50.0);
    }

    #[test]
    fn test_cross_shard_transaction() {
        let manager = ShardingManager::new(4);
        let from_address = "0x3333333333333333333333333333333333333333".to_string();
        let to_address = "0x4444444444444444444444444444444444444444".to_string();

        manager.initialize_balance(&from_address, &CurrencyType::BasicNeeds, 100.0).unwrap();

        let transaction = Transaction {
            from: from_address.clone(),
            to: to_address.clone(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        assert!(manager.process_transaction(&transaction).is_ok());

        assert_eq!(manager.get_balance(&from_address, &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(manager.get_balance(&to_address, &CurrencyType::BasicNeeds).unwrap(), 50.0);
    }

    #[test]
    fn test_resize_shards() {
        let mut manager = ShardingManager::new(2);
        let address1 = "0x5555555555555555555555555555555555555555".to_string();
        let address2 = "0x6666666666666666666666666666666666666666".to_string();

        manager.initialize_balance(&address1, &CurrencyType::BasicNeeds, 100.0).unwrap();
        manager.initialize_balance(&address2, &CurrencyType::Education, 200.0).unwrap();

        manager.resize_shards(4).unwrap();

        assert_eq!(manager.shard_count, 4);
        
        // Check if balances are preserved after resizing
        assert_eq!(manager.get_balance(&address1, &CurrencyType::BasicNeeds).unwrap(), 100.0);
        assert_eq!(manager.get_balance(&address2, &CurrencyType::Education).unwrap(), 200.0);

        // Check if addresses are properly redistributed
        let shard_addresses: Vec<Vec<String>> = (0..4)
            .map(|i| manager.get_shard_addresses(i).unwrap())
            .collect();

        assert!(shard_addresses.iter().any(|addrs| addrs.contains(&address1)));
        assert!(shard_addresses.iter().any(|addrs| addrs.contains(&address2)));

        // Ensure no address is in multiple shards
        let all_addresses: std::collections::HashSet<_> = shard_addresses.iter().flatten().collect();
        assert_eq!(all_addresses.len(), 2);
    }

    #[test]
    fn test_get_shard_currencies() {
        let manager = ShardingManager::new(2);
        let address = "0x7777777777777777777777777777777777777777".to_string();

        manager.initialize_balance(&address, &CurrencyType::BasicNeeds, 100.0).unwrap();
        manager.initialize_balance(&address, &CurrencyType::Education, 50.0).unwrap();

        let shard_id = manager.get_shard_for_address(&address);
        let currencies = manager.get_shard_currencies(shard_id).unwrap();

        assert_eq!(currencies.len(), 2);
        assert!(currencies.contains(&CurrencyType::BasicNeeds));
        assert!(currencies.contains(&CurrencyType::Education));
    }

    #[test]
    fn test_insufficient_balance() {
        let manager = ShardingManager::new(2);
        let from_address = "0x8888888888888888888888888888888888888888".to_string();
        let to_address = "0x9999999999999999999999999999999999999999".to_string();

        manager.initialize_balance(&from_address, &CurrencyType::BasicNeeds, 50.0).unwrap();

        let transaction = Transaction {
            from: from_address.clone(),
            to: to_address,
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        assert!(manager.process_transaction(&transaction).is_err());
        assert_eq!(manager.get_balance(&from_address, &CurrencyType::BasicNeeds).unwrap(), 50.0);
    }

    #[test]
    #[should_panic(expected = "Invalid shard ID")]
    fn test_invalid_shard_id() {
        let manager = ShardingManager::new(2);
        manager.get_shard_transactions(2).unwrap();
    }
}