// File: icn_sharding/src/lib.rs

use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use log::{info, warn, error};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardData {
    pub transactions: Vec<Transaction>,
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

pub struct ShardingManager {
    shard_count: u64,
    address_to_shard: Arc<RwLock<HashMap<String, u64>>>,
    shard_data: Arc<RwLock<Vec<ShardData>>>,
}

impl ShardingManager {
    pub fn new(shard_count: u64) -> Self {
        let mut shard_data = Vec::with_capacity(shard_count as usize);
        for _ in 0..shard_count {
            shard_data.push(ShardData {
                transactions: Vec::new(),
                balances: HashMap::new(),
            });
        }

        ShardingManager {
            shard_count,
            address_to_shard: Arc::new(RwLock::new(HashMap::new())),
            shard_data: Arc::new(RwLock::new(shard_data)),
        }
    }

    pub fn get_shard_count(&self) -> u64 {
        self.shard_count
    }

    pub fn get_shard_for_address(&self, address: &str) -> u64 {
        let address_to_shard = self.address_to_shard.read().unwrap();
        *address_to_shard.get(address).unwrap_or(&(self.hash_address(address) % self.shard_count))
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
        let mut shard_data = self.shard_data.write().unwrap();
        let shard = &mut shard_data[shard_id as usize];

        let from_balance = shard.balances
            .entry(transaction.from.clone())
            .or_default()
            .entry(transaction.currency_type.clone())
            .or_insert(0.0);

        if *from_balance < transaction.amount {
            return Err(IcnError::Sharding("Insufficient balance".into()));
        }

        *from_balance -= transaction.amount;
        *shard.balances
            .entry(transaction.to.clone())
            .or_default()
            .entry(transaction.currency_type.clone())
            .or_insert(0.0) += transaction.amount;

        shard.transactions.push(transaction.clone());

        Ok(())
    }

    fn process_cross_shard_transaction(&self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> IcnResult<()> {
        self.lock_funds(from_shard, &transaction.from, transaction.amount, &transaction.currency_type)?;
        self.transfer_between_shards(from_shard, to_shard, transaction)?;
        Ok(())
    }

    fn lock_funds(&self, shard_id: u64, address: &str, amount: f64, currency_type: &CurrencyType) -> IcnResult<()> {
        let mut shard_data = self.shard_data.write().unwrap();
        let shard = &mut shard_data[shard_id as usize];

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

    pub fn transfer_between_shards(&self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> IcnResult<()> {
        let mut shard_data = self.shard_data.write().unwrap();

        let to_shard_data = &mut shard_data[to_shard as usize];
        *to_shard_data.balances
            .entry(transaction.to.clone())
            .or_default()
            .entry(transaction.currency_type.clone())
            .or_insert(0.0) += transaction.amount;

        shard_data[from_shard as usize].transactions.push(transaction.clone());
        shard_data[to_shard as usize].transactions.push(transaction.clone());

        Ok(())
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let shard_id = self.get_shard_for_address(address);
        let shard_data = self.shard_data.read().unwrap();
        let shard = &shard_data[shard_id as usize];
        
        Ok(shard.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .cloned()
            .unwrap_or(0.0))
    }

    pub fn get_total_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let mut total_balance = 0.0;
        let shard_data = self.shard_data.read().unwrap();
        
        for shard in shard_data.iter() {
            if let Some(balances) = shard.balances.get(address) {
                if let Some(balance) = balances.get(currency_type) {
                    total_balance += balance;
                }
            }
        }

        Ok(total_balance)
    }

    pub fn initialize_balance(&self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let shard_id = self.get_shard_for_address(address);
        let mut shard_data = self.shard_data.write().unwrap();
        let shard = &mut shard_data[shard_id as usize];

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

        let shard_data = self.shard_data.read().unwrap();
        Ok(shard_data[shard_id as usize].transactions.clone())
    }

    pub fn get_shard_addresses(&self, shard_id: u64) -> IcnResult<Vec<String>> {
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding(format!("Invalid shard ID: {}", shard_id)));
        }

        let shard_data = self.shard_data.read().unwrap();
        Ok(shard_data[shard_id as usize].balances.keys().cloned().collect())
    }

    pub fn get_shard_currencies(&self, shard_id: u64) -> IcnResult<Vec<CurrencyType>> {
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding(format!("Invalid shard ID: {}", shard_id)));
        }

        let shard_data = self.shard_data.read().unwrap();
        let mut currencies = HashSet::new();
        for balances in shard_data[shard_id as usize].balances.values() {
            currencies.extend(balances.keys().cloned());
        }

        Ok(currencies.into_iter().collect())
    }

    pub fn resize_shards(&mut self, new_shard_count: u64) -> IcnResult<()> {
        if new_shard_count == 0 {
            return Err(IcnError::Sharding("Shard count must be greater than zero".into()));
        }

        let mut new_shard_data = Vec::with_capacity(new_shard_count as usize);
        for _ in 0..new_shard_count {
            new_shard_data.push(ShardData {
                transactions: Vec::new(),
                balances: HashMap::new(),
            });
        }

        let old_shard_data = std::mem::replace(&mut *self.shard_data.write().unwrap(), new_shard_data);

        // Redistribute balances and transactions
        for (old_shard_id, old_shard) in old_shard_data.into_iter().enumerate() {
            for (address, balances) in old_shard.balances {
                let new_shard_id = self.hash_address(&address) % new_shard_count;
                let new_shard = &mut self.shard_data.write().unwrap()[new_shard_id as usize];
                new_shard.balances.insert(address, balances);
            }

            for transaction in old_shard.transactions {
                let new_shard_id = self.get_shard_for_address(&transaction.from);
                let new_shard = &mut self.shard_data.write().unwrap()[new_shard_id as usize];
                new_shard.transactions.push(transaction);
            }
        }

        self.shard_count = new_shard_count;
        Ok(())
    }

    fn hash_address(&self, address: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        address.hash(&mut hasher);
        hasher.finish()
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

        manager.address_to_shard.write().unwrap().insert(address.clone(), 2);
        assert_eq!(manager.get_shard_for_address(&address), 2);
    }

    #[test]
    fn test_intra_shard_transaction() {
        let manager = ShardingManager::new(4);
        let from_address = "0x1111111111111111111111111111111111111111".to_string();
        let to_address = "0x2222222222222222222222222222222222222222".to_string();
        manager.address_to_shard.write().unwrap().insert(from_address.clone(), 1);
        manager.address_to_shard.write().unwrap().insert(to_address.clone(), 1);

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
        manager.address_to_shard.write().unwrap().insert(from_address.clone(), 1);
        manager.address_to_shard.write().unwrap().insert(to_address.clone(), 2);

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
    fn test_get_total_balance() {
        let manager = ShardingManager::new(4);
        let address = "0x5555555555555555555555555555555555555555".to_string();

        manager.initialize_balance(&address, &CurrencyType::BasicNeeds, 25.0).unwrap();
        manager.initialize_balance(&address, &CurrencyType::Education, 50.0).unwrap();

        assert_eq!(manager.get_total_balance(&address, &CurrencyType::BasicNeeds).unwrap(), 25.0);
        assert_eq!(manager.get_total_balance(&address, &CurrencyType::Education).unwrap(), 50.0);
    }

    #[test]
    fn test_resize_shards() {
        let mut manager = ShardingManager::new(2);
        let address1 = "0x6666666666666666666666666666666666666666".to_string();
        let address2 = "0x7777777777777777777777777777777777777777".to_string();

        manager.initialize_balance(&address1, &CurrencyType::BasicNeeds, 100.0).unwrap();
        manager.initialize_balance(&address2, &CurrencyType::Education, 200.0).unwrap();

        manager.resize_shards(4).unwrap();

        assert_eq!(manager.get_shard_count(), 4);
        
        // Check if balances are preserved after resizing
        assert_eq!(manager.get_total_balance(&address1, &CurrencyType::BasicNeeds).unwrap(), 100.0);
        assert_eq!(manager.get_total_balance(&address2, &CurrencyType::Education).unwrap(), 200.0);
    }

    #[test]
    fn test_get_shard_transactions() {
        let manager = ShardingManager::new(2);
        let address1 = "0x8888888888888888888888888888888888888888".to_string();
        let address2 = "0x9999999999999999999999999999999999999999".to_string();

        manager.address_to_shard.write().unwrap().insert(address1.clone(), 0);
        manager.address_to_shard.write().unwrap().insert(address2.clone(), 0);

        manager.initialize_balance(&address1, &CurrencyType::BasicNeeds, 100.0).unwrap();

        let transaction = Transaction {
            from: address1.clone(),
            to: address2.clone(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        manager.process_transaction(&transaction).unwrap();

        let shard_transactions = manager.get_shard_transactions(0).unwrap();
        assert_eq!(shard_transactions.len(), 1);
        assert_eq!(shard_transactions[0].from, address1);
        assert_eq!(shard_transactions[0].to, address2);
    }

    #[test]
    fn test_get_shard_addresses() {
        let manager = ShardingManager::new(2);
        let address1 = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
        let address2 = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();

        manager.address_to_shard.write().unwrap().insert(address1.clone(), 0);
        manager.address_to_shard.write().unwrap().insert(address2.clone(), 0);

        manager.initialize_balance(&address1, &CurrencyType::BasicNeeds, 100.0).unwrap();
        manager.initialize_balance(&address2, &CurrencyType::BasicNeeds, 100.0).unwrap();

        let shard_addresses = manager.get_shard_addresses(0).unwrap();
        assert_eq!(shard_addresses.len(), 2);
        assert!(shard_addresses.contains(&address1));
        assert!(shard_addresses.contains(&address2));
    }

    #[test]
    fn test_get_shard_currencies() {
        let manager = ShardingManager::new(2);
        let address = "0xcccccccccccccccccccccccccccccccccccccccc".to_string();

        manager.address_to_shard.write().unwrap().insert(address.clone(), 0);
        manager.initialize_balance(&address, &CurrencyType::BasicNeeds, 100.0).unwrap();
        manager.initialize_balance(&address, &CurrencyType::Education, 50.0).unwrap();

        let shard_currencies = manager.get_shard_currencies(0).unwrap();
        assert_eq!(shard_currencies.len(), 2);
        assert!(shard_currencies.contains(&CurrencyType::BasicNeeds));
        assert!(shard_currencies.contains(&CurrencyType::Education));
    }

    #[test]
    fn test_insufficient_balance() {
        let manager = ShardingManager::new(2);
        let address1 = "0xdddddddddddddddddddddddddddddddddddddddd".to_string();
        let address2 = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string();

        manager.address_to_shard.write().unwrap().insert(address1.clone(), 0);
        manager.address_to_shard.write().unwrap().insert(address2.clone(), 0);

        manager.initialize_balance(&address1, &CurrencyType::BasicNeeds, 50.0).unwrap();

        let transaction = Transaction {
            from: address1.clone(),
            to: address2.clone(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        assert!(manager.process_transaction(&transaction).is_err());
    }

    #[test]
    #[should_panic(expected = "Invalid shard ID")]
    fn test_invalid_shard_id() {
        let manager = ShardingManager::new(2);
        manager.get_shard_transactions(2).unwrap();
    }
}