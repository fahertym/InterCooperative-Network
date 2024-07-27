use icn_common::{IcnResult, IcnError, Transaction, Block, CurrencyType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use sha2::{Sha256, Digest};

pub struct ShardingManager {
    shard_count: u64,
    address_to_shard: Arc<RwLock<HashMap<String, u64>>>,
    shard_data: Arc<RwLock<Vec<ShardData>>>,
}

struct ShardData {
    transactions: Vec<Transaction>,
    balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

#[derive(Clone, Debug)]
pub struct CrossShardTransaction {
    pub transaction: Transaction,
    pub from_shard: u64,
    pub to_shard: u64,
    pub status: CrossShardTransactionStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CrossShardTransactionStatus {
    Initiated,
    LockAcquired,
    Committed,
    Failed(String),
}

impl ShardingManager {
    pub fn new(shard_count: u64) -> Self {
        let mut shard_data = Vec::new();
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

    pub fn get_shard_for_address(&self, address: &str) -> u64 {
        let address_to_shard = self.address_to_shard.read().unwrap();
        *address_to_shard.get(address).unwrap_or(&(self.hash_address(address) % self.shard_count))
    }

    pub fn add_address_to_shard(&self, address: String, shard_id: u64) -> IcnResult<()> {
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".into()));
        }
        let mut address_to_shard = self.address_to_shard.write().unwrap();
        address_to_shard.insert(address, shard_id);
        Ok(())
    }

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
            .or_insert_with(HashMap::new)
            .entry(transaction.currency_type.clone())
            .or_insert(0.0);

        if *from_balance < transaction.amount {
            return Err(IcnError::Sharding("Insufficient balance".into()));
        }

        *from_balance -= transaction.amount;
        *shard.balances
            .entry(transaction.to.clone())
            .or_insert_with(HashMap::new)
            .entry(transaction.currency_type.clone())
            .or_insert(0.0) += transaction.amount;

        shard.transactions.push(transaction);

        Ok(())
    }

    fn process_cross_shard_transaction(&self, from_shard: u64, to_shard: u64, transaction: Transaction) -> IcnResult<()> {
        let cross_shard_tx = CrossShardTransaction {
            transaction: transaction.clone(),
            from_shard,
            to_shard,
            status: CrossShardTransactionStatus::Initiated,
        };

        self.lock_funds(from_shard, &transaction.from, transaction.amount, &transaction.currency_type)?;

        let mut cross_shard_tx = cross_shard_tx;
        cross_shard_tx.status = CrossShardTransactionStatus::LockAcquired;

        self.transfer_between_shards(from_shard, to_shard, &transaction)?;

        cross_shard_tx.status = CrossShardTransactionStatus::Committed;

        Ok(())
    }

    fn lock_funds(&self, shard_id: u64, address: &str, amount: f64, currency_type: &CurrencyType) -> IcnResult<()> {
        let mut shard_data = self.shard_data.write().unwrap();
        let shard = &mut shard_data[shard_id as usize];

        let balance = shard.balances
            .entry(address.to_string())
            .or_insert_with(HashMap::new)
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
            .or_insert_with(HashMap::new)
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

    pub fn get_total_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let mut total_balance = 0.0;
        for shard_id in 0..self.shard_count {
            total_balance += self.get_shard_balance(shard_id, address, currency_type)?;
        }
        Ok(total_balance)
    }

    pub fn create_shard_block(&self, shard_id: u64) -> IcnResult<Block> {
        let mut shard_data = self.shard_data.write().unwrap();
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".into()));
        }

        let shard = &mut shard_data[shard_id as usize];
        let transactions = std::mem::take(&mut shard.transactions);

        let block = Block {
            index: 0,
            timestamp: chrono::Utc::now().timestamp(),
            transactions,
            previous_hash: "0".to_string(),
            hash: "0".to_string(),
        };

        Ok(block)
    }

    pub fn apply_block_to_shard(&self, shard_id: u64, block: &Block) -> IcnResult<()> {
        let mut shard_data = self.shard_data.write().unwrap();
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".into()));
        }

        let shard = &mut shard_data[shard_id as usize];

        for transaction in &block.transactions {
            let from_balance = shard.balances
                .entry(transaction.from.clone())
                .or_insert_with(HashMap::new)
                .entry(transaction.currency_type.clone())
                .or_insert(0.0);

            *from_balance -= transaction.amount;

            let to_balance = shard.balances
                .entry(transaction.to.clone())
                .or_insert_with(HashMap::new)
                .entry(transaction.currency_type.clone())
                .or_insert(0.0);

            *to_balance += transaction.amount;
        }

        Ok(())
    }

    pub fn get_shard_count(&self) -> u64 {
        self.shard_count
    }

    pub fn get_shard_transactions(&self, shard_id: u64) -> IcnResult<Vec<Transaction>> {
        let shard_data = self.shard_data.read().unwrap();
        if shard_id >= self.shard_count {
            return Err(IcnError::Sharding("Invalid shard ID".into()));
        }

        Ok(shard_data[shard_id as usize].transactions.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::CurrencyType;

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
    fn test_create_and_apply_shard_block() {
        let manager = ShardingManager::new(4);
        let address1 = "0x5555555555555555555555555555555555555555".to_string();
        let address2 = "0x6666666666666666666666666666666666666666".to_string();
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

        assert!(manager.process_transaction(transaction).is_ok());

        let block = manager.create_shard_block(1).unwrap();
        assert_eq!(block.transactions.len(), 1);

        assert!(manager.apply_block_to_shard(1, &block).is_ok());

        assert_eq!(manager.get_shard_balance(1, &address1, &CurrencyType::BasicNeeds).unwrap(), 50.0);
        assert_eq!(manager.get_shard_balance(1, &address2, &CurrencyType::BasicNeeds).unwrap(), 50.0);

        let shard_transactions = manager.get_shard_transactions(1).unwrap();
        assert_eq!(shard_transactions.len(), 0);
    }

    #[test]
    fn test_get_total_balance() {
        let manager = ShardingManager::new(4);
        let address = "0x7777777777777777777777777777777777777777".to_string();

        {
            let mut shard_data = manager.shard_data.write().unwrap();
            shard_data[0].balances.insert(address.clone(), HashMap::new());
            shard_data[0].balances.get_mut(&address).unwrap().insert(CurrencyType::BasicNeeds, 100.0);
            shard_data[1].balances.insert(address.clone(), HashMap::new());
            shard_data[1].balances.get_mut(&address).unwrap().insert(CurrencyType::BasicNeeds, 150.0);
            shard_data[2].balances.insert(address.clone(), HashMap::new());
            shard_data[2].balances.get_mut(&address).unwrap().insert(CurrencyType::BasicNeeds, 200.0);
        }

        let total_balance = manager.get_total_balance(&address, &CurrencyType::BasicNeeds).unwrap();
        assert_eq!(total_balance, 450.0);
    }

    #[test]
    fn test_insufficient_balance() {
        let manager = ShardingManager::new(4);
        let from_address = "0x8888888888888888888888888888888888888888".to_string();
        let to_address = "0x9999999999999999999999999999999999999999".to_string();
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
            amount: 150.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        assert!(manager.process_transaction(transaction).is_err());

        assert_eq!(manager.get_shard_balance(1, &from_address, &CurrencyType::BasicNeeds).unwrap(), 100.0);
        assert_eq!(manager.get_shard_balance(1, &to_address, &CurrencyType::BasicNeeds).unwrap(), 0.0);
    }

    #[test]
    fn test_invalid_shard_id() {
        let manager = ShardingManager::new(4);
        let address = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();

        assert!(manager.get_shard_balance(4, &address, &CurrencyType::BasicNeeds).is_err());
        assert!(manager.create_shard_block(4).is_err());
        assert!(manager.apply_block_to_shard(4, &Block {
            index: 0,
            timestamp: 0,
            transactions: vec![],
            previous_hash: "0".to_string(),
            hash: "0".to_string(),
        }).is_err());
    }
}
