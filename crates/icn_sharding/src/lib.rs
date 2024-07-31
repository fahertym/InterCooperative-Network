// File: crates/icn_sharding/src/lib.rs

use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use sha2::{Sha256, Digest};
use log::{info, warn, error};

/// Represents the sharding manager for the InterCooperative Network.
pub struct ShardingManager {
    shard_count: u64,
    address_to_shard: Arc<RwLock<HashMap<String, u64>>>,
    shard_data: Arc<RwLock<Vec<ShardData>>>,
}

/// Holds the transaction data and balances for each shard.
struct ShardData {
    transactions: Vec<Transaction>,
    balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

impl ShardingManager {
    /// Creates a new ShardingManager with the specified number of shards.
    ///
    /// # Arguments
    ///
    /// * `shard_count` - The number of shards to create.
    ///
    /// # Returns
    ///
    /// A new instance of ShardingManager.
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

    /// Determines the shard for a given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to determine the shard for.
    ///
    /// # Returns
    ///
    /// The shard ID for the given address.
    pub fn get_shard_for_address(&self, address: &str) -> u64 {
        let address_to_shard = self.address_to_shard.read().unwrap();
        *address_to_shard.get(address).unwrap_or(&(self.hash_address(address) % self.shard_count))
    }

    /// Adds an address to a specific shard.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to add.
    /// * `shard_id` - The ID of the shard to add the address to.
    ///
    /// # Returns
    ///
    /// An IcnResult indicating success or failure.
    pub fn add_address_to_shard(&self, address: String, shard_id: u64) -> IcnResult<()> {
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding(format!("Invalid shard ID: {}", shard_id)));
        }
        let mut address_to_shard = self.address_to_shard.write().unwrap();
        address_to_shard.insert(address, shard_id);
        Ok(())
    }

    /// Processes a transaction within the sharding system.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to process.
    ///
    /// # Returns
    ///
    /// An IcnResult indicating success or failure.
    pub fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        let from_shard = self.get_shard_for_address(&transaction.from);
        let to_shard = self.get_shard_for_address(&transaction.to);

        if from_shard == to_shard {
            self.process_intra_shard_transaction(from_shard, transaction)
        } else {
            self.process_cross_shard_transaction(from_shard, to_shard, transaction)
        }
    }

    fn process_intra_shard_transaction(&self, shard_id: u64, transaction: Transaction) -> IcnResult<()> {
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

        shard.transactions.push(transaction);

        Ok(())
    }

    fn process_cross_shard_transaction(&self, from_shard: u64, to_shard: u64, transaction: Transaction) -> IcnResult<()> {
        self.lock_funds(from_shard, &transaction.from, transaction.amount, &transaction.currency_type)?;
        self.transfer_between_shards(from_shard, to_shard, &transaction)?;
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

    fn transfer_between_shards(&self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> IcnResult<()> {
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

    fn hash_address(&self, address: &str) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(address.as_bytes());
        let result = hasher.finalize();
        let hash_bytes: [u8; 8] = result[..8].try_into().unwrap();
        u64::from_le_bytes(hash_bytes)
    }

    /// Gets the balance for a given address and currency type within a specific shard.
    ///
    /// # Arguments
    ///
    /// * `shard_id` - The ID of the shard to check.
    /// * `address` - The address to check the balance for.
    /// * `currency_type` - The type of currency to check the balance for.
    ///
    /// # Returns
    ///
    /// An IcnResult containing the balance if successful.
    pub fn get_shard_balance(&self, shard_id: u64, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let shard_data = self.shard_data.read().unwrap();
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".into()));
        }

        let shard = &shard_data[shard_id as usize];
        Ok(*shard.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .unwrap_or(&0.0))
    }

    /// Gets the total balance across all shards for a given address and currency type.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to check the balance for.
    /// * `currency_type` - The type of currency to check the balance for.
    ///
    /// # Returns
    ///
    /// An IcnResult containing the total balance if successful.
    pub fn get_total_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let mut total_balance = 0.0;
        for shard_id in 0..self.shard_count {
            total_balance += self.get_shard_balance(shard_id, address, currency_type)?;
        }
        Ok(total_balance)
    }

    /// Creates a new block for a specific shard.
    ///
    /// # Arguments
    ///
    /// * `shard_id` - The ID of the shard to create a block for.
    ///
    /// # Returns
    ///
    /// An IcnResult containing the list of transactions in the new block if successful.
    pub fn create_shard_block(&self, shard_id: u64) -> IcnResult<Vec<Transaction>> {
        let mut shard_data = self.shard_data.write().unwrap();
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".into()));
        }

        let shard = &mut shard_data[shard_id as usize];
        let transactions = std::mem::take(&mut shard.transactions);
        Ok(transactions)
    }

    /// Applies a block of transactions to a specific shard.
    ///
    /// # Arguments
    ///
    /// * `shard_id` - The ID of the shard to apply the block to.
    /// * `transactions` - The list of transactions in the block.
    ///
    /// # Returns
    ///
    /// An IcnResult indicating success or failure.
    pub fn apply_block_to_shard(&self, shard_id: u64, transactions: Vec<Transaction>) -> IcnResult<()> {
        let mut shard_data = self.shard_data.write().unwrap();
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".into()));
        }

        let shard = &mut shard_data[shard_id as usize];

        for transaction in transactions {
            let from_balance = shard.balances
                .entry(transaction.from.clone())
                .or_default()
                .entry(transaction.currency_type.clone())
                .or_insert(0.0);

            *from_balance -= transaction.amount;

            let to_balance = shard.balances
                .entry(transaction.to.clone())
                .or_default()
                .entry(transaction.currency_type.clone())
                .or_insert(0.0);

            *to_balance += transaction.amount;
        }

        Ok(())
    }

    /// Gets the number of shards in the system.
    ///
    /// # Returns
    ///
    /// The number of shards.
    pub fn get_shard_count(&self) -> u64 {
        self.shard_count
    }

    /// Gets all transactions for a specific shard.
    ///
    /// # Arguments
    ///
    /// * `shard_id` - The ID of the shard to get transactions for.
    ///
    /// # Returns
    ///
    /// An IcnResult containing the list of transactions if successful.
    pub fn get_shard_transactions(&self, shard_id: u64) -> IcnResult<Vec<Transaction>> {
        let shard_data = self.shard_data.read().unwrap();
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".into()));
        }

        Ok(shard_data[shard_id as usize].transactions.clone())
    }

    /// Gets all addresses present in a specific shard.
    ///
    /// # Arguments
    ///
    /// * `shard_id` - The ID of the shard to get addresses for.
    ///
    /// # Returns
    ///
    /// An IcnResult containing the list of addresses if successful.
    pub fn get_shard_addresses(&self, shard_id: u64) -> IcnResult<Vec<String>> {
        let shard_data = self.shard_data.read().unwrap();
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".into()));
        }

        Ok(shard_data[shard_id as usize].balances.keys().cloned().collect())
    }

    /// Gets all currency types present in a specific shard.
    ///
    /// # Arguments
    ///
    /// * `shard_id` - The ID of the shard to get currency types for.
    ///
    /// # Returns
    ///
    /// An IcnResult containing the list of currency types if successful.
    pub fn get_shard_currencies(&self, shard_id: u64) -> IcnResult<Vec<CurrencyType>> {
        let shard_data = self.shard_data.read().unwrap();
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".into()));
        }

        let mut currencies = HashSet::new();
        for balances in shard_data[shard_id as usize].balances.values() {
            currencies.extend(balances.keys().cloned());
        }

        Ok(currencies.into_iter().collect())
    }

    /// Resizes the number of shards in the system.
    ///
    /// # Arguments
    ///
    /// * `new_shard_count` - The new number of shards.
    ///
    /// # Returns
    ///
    /// An IcnResult indicating success or failure.
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

        manager.add_address_to_shard(address.clone(), 2).unwrap();
        assert_eq!(manager.get_shard_for_address(&address), 2);
    }

    #[test]
    fn test_intra_shard_transaction() {
        let manager = ShardingManager::new(4);
        let from_address = "0x1111111111111111111111111111111111111111".to_string();
        let to_address = "0x2222222222222222222222222222222222222222".to_string();
        manager.add_address_to_shard(from_address.clone(), 1).unwrap();
        manager.add_address_to_shard(to_address.clone(), 1).unwrap();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[1].balances.insert(from_address.clone(), HashMap::new());
            shard_data[1].balances.get_mut(&from_address).unwrap().insert(CurrencyType::BasicNeeds, 100.0);
        }

        let transaction = Transaction {
            from: from_address.clone(),
            to: to_address.clone(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        assert!(manager.process_transaction(transaction).is_ok());

        assert_eq!(manager.get_shard_balance(1, &from_address, &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(manager.get_shard_balance(1, &to_address, &CurrencyType::BasicNeeds).unwrap(), 50.0);
    }

    #[test]
    fn test_cross_shard_transaction() {
        let manager = ShardingManager::new(4);
        let from_address = "0x3333333333333333333333333333333333333333".to_string();
        let to_address = "0x4444444444444444444444444444444444444444".to_string();
        manager.add_address_to_shard(from_address.clone(), 1).unwrap();
        manager.add_address_to_shard(to_address.clone(), 2).unwrap();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[1].balances.insert(from_address.clone(), HashMap::new());
            shard_data[1].balances.get_mut(&from_address).unwrap().insert(CurrencyType::BasicNeeds, 100.0);
        }

        let transaction = Transaction {
            from: from_address.clone(),
            to: to_address.clone(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        assert!(manager.process_transaction(transaction).is_ok());

        assert_eq!(manager.get_shard_balance(1, &from_address, &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(manager.get_shard_balance(2, &to_address, &CurrencyType::BasicNeeds).unwrap(), 50.0);
    }

    #[test]
    fn test_get_total_balance() {
        let manager = ShardingManager::new(4);
        let address = "0x5555555555555555555555555555555555555555".to_string();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            for i in 0..4 {
                shard_data[i].balances.insert(address.clone(), HashMap::new());
                shard_data[i].balances.get_mut(&address).unwrap().insert(CurrencyType::BasicNeeds, 25.0);
            }
        }

        assert_eq!(manager.get_total_balance(&address, &CurrencyType::BasicNeeds).unwrap(), 100.0);
    }

    #[test]
    fn test_create_and_apply_shard_block() {
        let manager = ShardingManager::new(4);
        let address1 = "0x6666666666666666666666666666666666666666".to_string();
        let address2 = "0x7777777777777777777777777777777777777777".to_string();
        manager.add_address_to_shard(address1.clone(), 1).unwrap();
        manager.add_address_to_shard(address2.clone(), 1).unwrap();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[1].balances.insert(address1.clone(), HashMap::new());
            shard_data[1].balances.get_mut(&address1).unwrap().insert(CurrencyType::BasicNeeds, 100.0);
        }

        let transaction = Transaction {
            from: address1.clone(),
            to: address2.clone(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        manager.process_transaction(transaction).unwrap();

        let block = manager.create_shard_block(1).unwrap();
        assert_eq!(block.len(), 1);

        // Reset the shard data
        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[1].balances.get_mut(&address1).unwrap().insert(CurrencyType::BasicNeeds, 100.0);
            shard_data[1].balances.get_mut(&address2).unwrap().remove(&CurrencyType::BasicNeeds);
        }

        manager.apply_block_to_shard(1, block).unwrap();

        assert_eq!(manager.get_shard_balance(1, &address1, &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(manager.get_shard_balance(1, &address2, &CurrencyType::BasicNeeds).unwrap(), 50.0);
    }

    #[test]
    fn test_get_shard_addresses() {
        let manager = ShardingManager::new(4);
        let address1 = "0x8888888888888888888888888888888888888888".to_string();
        let address2 = "0x9999999999999999999999999999999999999999".to_string();
        manager.add_address_to_shard(address1.clone(), 1).unwrap();
        manager.add_address_to_shard(address2.clone(), 1).unwrap();

        let addresses = manager.get_shard_addresses(1).unwrap();
        assert_eq!(addresses.len(), 2);
        assert!(addresses.contains(&address1));
        assert!(addresses.contains(&address2));
    }

    #[test]
    fn test_get_shard_currencies() {
        let manager = ShardingManager::new(4);
        let address = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
        manager.add_address_to_shard(address.clone(), 1).unwrap();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[1].balances.insert(address.clone(), HashMap::new());
            shard_data[1].balances.get_mut(&address).unwrap().insert(CurrencyType::BasicNeeds, 100.0);
            shard_data[1].balances.get_mut(&address).unwrap().insert(CurrencyType::Education, 50.0);
        }

        let currencies = manager.get_shard_currencies(1).unwrap();
        assert_eq!(currencies.len(), 2);
        assert!(currencies.contains(&CurrencyType::BasicNeeds));
        assert!(currencies.contains(&CurrencyType::Education));
    }

    #[test]
    fn test_resize_shards() {
        let mut manager = ShardingManager::new(4);
        let address1 = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
        let address2 = "0xcccccccccccccccccccccccccccccccccccccccc".to_string();
        manager.add_address_to_shard(address1.clone(), 1).unwrap();
        manager.add_address_to_shard(address2.clone(), 2).unwrap();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[1].balances.insert(address1.clone(), HashMap::new());
            shard_data[1].balances.get_mut(&address1).unwrap().insert(CurrencyType::BasicNeeds, 100.0);
            shard_data[2].balances.insert(address2.clone(), HashMap::new());
            shard_data[2].balances.get_mut(&address2).unwrap().insert(CurrencyType::Education, 50.0);
        }

        manager.resize_shards(2).unwrap();

        assert_eq!(manager.get_shard_count(), 2);
        
        // Check if balances are preserved after resizing
        let new_shard1 = manager.get_shard_for_address(&address1);
        let new_shard2 = manager.get_shard_for_address(&address2);
        
        assert_eq!(manager.get_shard_balance(new_shard1, &address1, &CurrencyType::BasicNeeds).unwrap(), 100.0);
        assert_eq!(manager.get_shard_balance(new_shard2, &address2, &CurrencyType::Education).unwrap(), 50.0);
    }

    #[test]
    fn test_invalid_shard_operations() {
        let manager = ShardingManager::new(4);
        
        // Test invalid shard ID
        assert!(manager.get_shard_balance(5, "address", &CurrencyType::BasicNeeds).is_err());
        assert!(manager.get_shard_transactions(5).is_err());
        assert!(manager.get_shard_addresses(5).is_err());
        assert!(manager.get_shard_currencies(5).is_err());
        
        // Test invalid address to shard assignment
        assert!(manager.add_address_to_shard("address".to_string(), 5).is_err());
    }

    #[test]
    fn test_insufficient_balance() {
        let manager = ShardingManager::new(4);
        let from_address = "0xdddddddddddddddddddddddddddddddddddddddd".to_string();
        let to_address = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string();
        manager.add_address_to_shard(from_address.clone(), 1).unwrap();
        manager.add_address_to_shard(to_address.clone(), 1).unwrap();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[1].balances.insert(from_address.clone(), HashMap::new());
            shard_data[1].balances.get_mut(&from_address).unwrap().insert(CurrencyType::BasicNeeds, 50.0);
        }

        let transaction = Transaction {
            from: from_address.clone(),
            to: to_address.clone(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        assert!(manager.process_transaction(transaction).is_err());
    }

    #[test]
    fn test_multiple_currencies() {
        let manager = ShardingManager::new(4);
        let address = "0xffffffffffffffffffffffffffffffffffffffff".to_string();
        manager.add_address_to_shard(address.clone(), 1).unwrap();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[1].balances.insert(address.clone(), HashMap::new());
            shard_data[1].balances.get_mut(&address).unwrap().insert(CurrencyType::BasicNeeds, 100.0);
            shard_data[1].balances.get_mut(&address).unwrap().insert(CurrencyType::Education, 50.0);
            shard_data[1].balances.get_mut(&address).unwrap().insert(CurrencyType::Environmental, 25.0);
        }

        assert_eq!(manager.get_shard_balance(1, &address, &CurrencyType::BasicNeeds).unwrap(), 100.0);
        assert_eq!(manager.get_shard_balance(1, &address, &CurrencyType::Education).unwrap(), 50.0);
        assert_eq!(manager.get_shard_balance(1, &address, &CurrencyType::Environmental).unwrap(), 25.0);

        let currencies = manager.get_shard_currencies(1).unwrap();
        assert_eq!(currencies.len(), 3);
    }

    #[test]
    fn test_shard_transactions() {
        let manager = ShardingManager::new(4);
        let address1 = "0x1111111111111111111111111111111111111111".to_string();
        let address2 = "0x2222222222222222222222222222222222222222".to_string();
        manager.add_address_to_shard(address1.clone(), 1).unwrap();
        manager.add_address_to_shard(address2.clone(), 1).unwrap();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[1].balances.insert(address1.clone(), HashMap::new());
            shard_data[1].balances.get_mut(&address1).unwrap().insert(CurrencyType::BasicNeeds, 100.0);
        }

        let transaction1 = Transaction {
            from: address1.clone(),
            to: address2.clone(),
            amount: 20.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        let transaction2 = Transaction {
            from: address1.clone(),
            to: address2.clone(),
            amount: 30.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 1,
            signature: None,
        };

        manager.process_transaction(transaction1.clone()).unwrap();
        manager.process_transaction(transaction2.clone()).unwrap();

        let transactions = manager.get_shard_transactions(1).unwrap();
        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0], transaction1);
        assert_eq!(transactions[1], transaction2);
    }

    #[test]
    fn test_resize_shards_preserves_data() {
        let mut manager = ShardingManager::new(2);
        let address1 = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
        let address2 = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();

        manager.add_address_to_shard(address1.clone(), 0).unwrap();
        manager.add_address_to_shard(address2.clone(), 1).unwrap();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[0].balances.insert(address1.clone(), HashMap::new());
            shard_data[0].balances.get_mut(&address1).unwrap().insert(CurrencyType::BasicNeeds, 50.0);
            shard_data[1].balances.insert(address2.clone(), HashMap::new());
            shard_data[1].balances.get_mut(&address2).unwrap().insert(CurrencyType::BasicNeeds, 100.0);
        }

        manager.resize_shards(4).unwrap();

        let new_shard_id1 = manager.get_shard_for_address(&address1);
        let new_shard_id2 = manager.get_shard_for_address(&address2);

        assert_eq!(manager.get_shard_balance(new_shard_id1, &address1, &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(manager.get_shard_balance(new_shard_id2, &address2, &CurrencyType::BasicNeeds).unwrap(), 100.0);
    }
}
